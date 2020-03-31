#include <spdlog/sinks/stdout_color_sinks.h>
#include <spdlog/spdlog.h>

#include <Porter2.hpp>
#include <binary_collection.hpp>
#include <binary_freq_collection.hpp>
#include <boost/algorithm/string.hpp>
#include <compress.hpp>
#include <fmt/format.h>
#include <forward_index_builder.hpp>
#include <invert.hpp>
#include <mappable/mapper.hpp>
#include <mio/mmap.hpp>
#include <nlohmann/json.hpp>
#include <tbb/task_scheduler_init.h>
#include <wand_data.hpp>
#include <wand_data_raw.hpp>

#include <recursive_graph_bisection.hpp>
#include <util/inverted_index_utils.hpp>
#include <util/progress.hpp>

static std::size_t const THREADS = std::thread::hardware_concurrency();
static std::size_t const BATCH_SIZE = 10'000;
static std::string const IDX_DIR = "idx";
static std::string const FWD = "fwd";
static std::string const INV = "inv";
static pisa::BlockSize const BLOCK_SIZE = pisa::FixedBlock(40);

using pisa::BlockSize;
using pisa::Document_Record;
using pisa::Forward_Index_Builder;

using Wand = pisa::wand_data<pisa::wand_data_raw>;

void parse()
{
    pisa::Forward_Index_Builder fwd_builder;
    fwd_builder.build(
        std::cin,
        fmt::format("{}/{}", IDX_DIR, FWD),
        [](std::istream& in) -> std::optional<Document_Record> {
            std::string line;
            if (std::getline(in, line) && not line.empty()) {
                auto record = nlohmann::basic_json<>::parse(line);
                return std::make_optional<Document_Record>(
                    record["id"].get<std::string>(), record["text"].get<std::string>(), "");
            }
            return std::nullopt;
        },
        [](std::string&& term) -> std::string {
            boost::algorithm::to_lower(term);
            return std::move(term);
        },
        pisa::parse_plaintext_content,
        BATCH_SIZE,
        THREADS);
}

void invert()
{
    auto term_lexicon_file = fmt::format("{}/{}.termlex", IDX_DIR, FWD);
    mio::mmap_source mfile(term_lexicon_file.c_str());
    auto lexicon = pisa::Payload_Vector<>::from(mfile);
    pisa::invert::invert_forward_index(
        fmt::format("{}/{}", IDX_DIR, FWD),
        fmt::format("{}/{}", IDX_DIR, INV),
        lexicon.size(),
        BATCH_SIZE,
        THREADS);
}

void bmw(pisa::binary_collection const& sizes, pisa::binary_freq_collection const& coll)
{
    Wand wdata(sizes.begin()->begin(), coll.num_docs(), coll, "bm25", BLOCK_SIZE, true, {});
    pisa::mapper::freeze(wdata, fmt::format("{}/{}.bm25.bmw", IDX_DIR, INV).c_str());
}

void compress()
{
    pisa::binary_collection sizes((fmt::format("{}/{}.bp.sizes", IDX_DIR, INV).c_str()));
    pisa::binary_freq_collection coll(fmt::format("{}/{}.bp", IDX_DIR, INV).c_str());
    bmw(sizes, coll);
    pisa::compress_index<pisa::block_simdbp_index, Wand>(
        coll,
        pisa::global_parameters{},
        fmt::format("{}/{}.simdbp", IDX_DIR, INV),
        false,
        "block_simdbp",
        fmt::format("{}/{}.bm25.bmw", IDX_DIR, INV),
        "bm25",
        true);
}

void reorder()
{
    pisa::forward_index fwd = pisa::forward_index::from_inverted_index(fmt::format("{}/{}", IDX_DIR, INV), 0, false);
    std::vector<uint32_t> documents(fwd.size());
    std::iota(documents.begin(), documents.end(), 0U);
    std::vector<double> gains(fwd.size(), 0.0);
    using iterator_type = std::vector<uint32_t>::iterator;
    using range_type = pisa::document_range<iterator_type>;
    range_type initial_range(documents.begin(), documents.end(), fwd, gains);
    auto depth = static_cast<size_t>(std::log2(fwd.size()) - 5);
    pisa::progress bp_progress("Graph bisection", initial_range.size() * depth);
    bp_progress.update(0);
    pisa::recursive_graph_bisection(initial_range, depth, depth - 6, bp_progress);
    auto mapping = pisa::get_mapping(documents);
    fwd.clear();
    documents.clear();
    pisa::reorder_inverted_index(fmt::format("{}/{}", IDX_DIR, INV), fmt::format("{}/{}.bp", IDX_DIR, INV), mapping);
}


int main(int argc, char const* argv[])
{
    spdlog::drop("");
    spdlog::set_default_logger(spdlog::stderr_color_mt(""));
    tbb::task_scheduler_init init(THREADS);
    parse();
    invert();
    reorder();
    compress();
}
