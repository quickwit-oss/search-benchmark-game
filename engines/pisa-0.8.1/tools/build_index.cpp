#include <spdlog/sinks/stdout_color_sinks.h>
#include <spdlog/spdlog.h>

#include <Porter2.hpp>
#include <boost/algorithm/string.hpp>
#include <fmt/format.h>
#include <forward_index_builder.hpp>
#include <invert.hpp>
#include <mio/mmap.hpp>
#include <nlohmann/json.hpp>
#include <tbb/task_scheduler_init.h>

static std::size_t const THREADS = 4;
static std::size_t const BATCH_SIZE = 10'000;
static std::string const FWD = "/tmp/fwd";
static std::string const INV = "/tmp/inv";

using pisa::Document_Record;
using pisa::Forward_Index_Builder;

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
                auto url = record["url"].get<std::string>();
                return std::make_optional<Document_Record>(
                    url,
                    fmt::format(
                        "{} {}", record["title"].get<std::string>(), record["url"].get<std::string>()),
                    url);
            }
            return std::nullopt;
        },
        [](std::string&& term) -> std::string {
            boost::algorithm::to_lower(term);
            return porter2::Stemmer{}.stem(term);
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

int main(int argc, char const* argv[])
{
    spdlog::drop("");
    spdlog::set_default_logger(spdlog::stderr_color_mt(""));

    tbb::task_scheduler_init init(THREADS);

    parse();
    invert();
}
