#include <iostream>

#include <boost/algorithm/string.hpp>
#include <cursor/block_max_scored_cursor.hpp>
#include <cursor/cursor.hpp>
#include <cursor/scored_cursor.hpp>
#include <index_types.hpp>
#include <mio/mmap.hpp>
#include <query/algorithm.hpp>
#include <query/queries.hpp>
#include <mappable/mapper.hpp>
#include <query/term_processor.hpp>
#include <scorer/scorer.hpp>
#include <spdlog/sinks/stdout_color_sinks.h>
#include <spdlog/spdlog.h>
#include <topk_queue.hpp>
#include <wand_data.hpp>
#include <wand_data_raw.hpp>

static std::string const IDX_DIR = "idx";
static std::string const FWD = "fwd";
static std::string const INV = "inv";

int main(int argc, char const* argv[])
{
    using namespace pisa;
    spdlog::drop("");
    spdlog::set_default_logger(spdlog::stderr_color_mt(""));

    std::string terms_file = fmt::format("{}/{}.termlex", IDX_DIR, FWD);
    std::string wand_data_filename = fmt::format("{}/{}.bm25.bmw", IDX_DIR, INV);
    std::string index_filename = fmt::format("{}/{}.simdbp", IDX_DIR, INV);

    std::string scorer_name = "quantized";

    auto term_processor = TermProcessor(terms_file, std::nullopt, std::nullopt);
    using wand_uniform_index_quantized = wand_data<wand_data_raw>;
    wand_uniform_index_quantized wdata;

    mio::mmap_source md;
    std::error_code error;
    md.map(wand_data_filename, error);
    if (error) {
        std::cerr << "error mapping file: " << error.message() << ", exiting..." << std::endl;
        throw std::runtime_error("Error opening file");
    }
    mapper::map(wdata, md, mapper::map_flags::warmup);

    using IndexType = block_simdbp_index;

    IndexType index;
    spdlog::info("Loading index from {}", index_filename);
    mio::mmap_source m(index_filename.c_str());
    mapper::map(index, m);

    auto scorer = scorer::from_name(scorer_name, wdata);


    std::vector<float> thresholds;
    std::ifstream tin(fmt::format("{}/{}.thresholds", IDX_DIR, FWD));
    std::string t;
    while (std::getline(tin, t)) {
        thresholds.push_back(std::stof(t));
    }


    std::string line;
    while (std::getline(std::cin, line)) {
        bool intersection = false;
        size_t count = 0;
        std::vector<std::string> tokens;
        boost::split(tokens, line, boost::is_any_of("\t"));
        if (boost::contains(tokens[1], "\"")) {
            std::cout << "UNSUPPORTED\n";
            continue;
        } else if (boost::starts_with(tokens[1], "+")) {
            intersection = true;
            boost::replace_all(tokens[1], "+", "");
        }


        Query query = parse_query_terms(tokens[1], term_processor);
        if (tokens[0] == "COUNT") {
            if(query.terms.size() == 1)
                count = index[query.terms[0]].size();
            else if (intersection) {
                and_query and_q;
                count = and_q(make_cursors(index, query), index.num_docs()).size();
            } else {
                or_query<false> or_q;
                count = or_q(make_cursors(index, query), index.num_docs());
            }
        } else if (tokens[0] == "TOP_10" || tokens[0] == "TOP_100" || tokens[0] == "TOP_1000") {
            size_t k;
            if (tokens[0] == "TOP_10") {
                k = 10;
            } else if (tokens[0] == "TOP_100") {
                k = 100;
            } else if (tokens[0] == "TOP_1000") {
                k = 1000;
            } else {
                throw std::runtime_error(fmt::format("Can't compute k for {}", tokens[0]));
            }
            topk_queue topk(k);
            if (intersection or query.terms.size() == 1) {
                ranked_and_query ranked_and_q(topk);
                ranked_and_q(make_scored_cursors(index, *scorer, query), index.num_docs());
                topk.finalize();
                count = 1;
            } else {
                float threshold = 0;
                for (auto &&term : query.terms){
                    threshold = std::max(threshold, thresholds[term]);
                }
                topk.set_threshold(threshold);
                block_max_wand_query block_max_wand_q(topk);
                block_max_wand_q(
                    make_block_max_scored_cursors(index, wdata, *scorer, query), index.num_docs());
                topk.finalize();
                count = 1;
            }
        } else if (tokens[0] == "TOP_10_COUNT" || tokens[0] == "TOP_100_COUNT" || tokens[0] == "TOP_1000_COUNT") {
            if (intersection or query.terms.size() == 1) {
                scored_and_query and_q;
                count = and_q(make_scored_cursors(index, *scorer, query), index.num_docs()).size();
            } else {
                or_query<true> or_q;
                count = or_q(make_cursors(index, query), index.num_docs());
            }
        } else {
            std::cout << "UNSUPPORTED\n";
        }
        std::cout << count << "\n";
    }
}
