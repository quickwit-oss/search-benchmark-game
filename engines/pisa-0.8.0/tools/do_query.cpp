#include <iostream>

#include <spdlog/sinks/stdout_color_sinks.h>
#include <spdlog/spdlog.h>
#include <boost/algorithm/string.hpp>

int main(int argc, char const *argv[])
{
    spdlog::drop("");
    spdlog::set_default_logger(spdlog::stderr_color_mt(""));

    std::string line;
    while (std::getline(std::cin, line))
    {
        std::vector<std::string> tokens;
        boost::split(tokens, line, boost::is_any_of("\t"));

        std::cout << "UNSUPPORTED" << std::endl;
    }

}