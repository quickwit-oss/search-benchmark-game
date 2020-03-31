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
#include <wand_data_compressed.hpp>

static std::size_t const THREADS = std::thread::hardware_concurrency();
static std::size_t const BATCH_SIZE = 10'000;
static std::string const FWD = "fwd";
static std::string const INV = "inv";
static pisa::BlockSize const BLOCK_SIZE = pisa::FixedBlock(128);

using pisa::BlockSize;
using pisa::Document_Record;
using pisa::Forward_Index_Builder;

using Wand = pisa::wand_data<pisa::wand_data_compressed<pisa::PayloadType::Quantized>>;

void parse()
{
    pisa::Forward_Index_Builder fwd_builder;
    fwd_builder.build(
        std::cin,
        FWD,
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
    auto term_lexicon_file = fmt::format("{}.termlex", FWD);
    mio::mmap_source mfile(term_lexicon_file.c_str());
    auto lexicon = pisa::Payload_Vector<>::from(mfile);
    pisa::invert::invert_forward_index(FWD, INV, lexicon.size(), BATCH_SIZE, THREADS);
}

void bmw(pisa::binary_collection const& sizes, pisa::binary_freq_collection const& coll)
{
    Wand wdata(sizes.begin()->begin(), coll.num_docs(), coll, "bm25", BLOCK_SIZE, false, {});
    pisa::mapper::freeze(wdata, fmt::format("{}.bm25.bmw", INV).c_str());
}

void compress()
{
    pisa::binary_collection sizes((fmt::format("{}.sizes", INV).c_str()));
    pisa::binary_freq_collection coll(INV.c_str());
    bmw(sizes, coll);
    pisa::compress_index<pisa::block_simdbp_index, Wand>(
        coll,
        pisa::global_parameters{},
        fmt::format("{}.simdbp", INV),
        true,
        "block_simdbp",
        fmt::format("{}.bm25.bmw", INV),
        "bm25",
        true);
}

int main(int argc, char const* argv[])
{
    spdlog::drop("");
    spdlog::set_default_logger(spdlog::stderr_color_mt(""));
    tbb::task_scheduler_init init(THREADS);
    parse();
    invert();
    compress();
}
