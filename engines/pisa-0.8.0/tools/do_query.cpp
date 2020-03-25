#include <iostream>

#include <spdlog/sinks/stdout_color_sinks.h>
#include <spdlog/spdlog.h>
#include <boost/algorithm/string.hpp>
#include <query/queries.hpp>
#include <query/term_processor.hpp>

int main(int argc, char const *argv[])
{
    spdlog::drop("");
    spdlog::set_default_logger(spdlog::stderr_color_mt(""));

    std::string terms_file;
    std::string stemmer = "porter2";

    auto term_processor = pisa::TermProcessor(terms_file, std::nullopt, stemmer);
    std::string line;
    while (std::getline(std::cin, line))
    {
        size_t count = 0;
        std::vector<std::string> tokens;
        boost::split(tokens, line, boost::is_any_of("\t"));
        pisa::Query q = pisa::parse_query_terms(tokens[1], term_processor);
        if(tokens[0] == "COUNT"){

        } else if(tokens[0] == "TOP_10"){
        } else if(tokens[0] == "TOP_10_COUNT"){

        }else{
            std::cout << "UNSUPPORTED\n";
        }
        std::cout << count << "\n";
    }

}