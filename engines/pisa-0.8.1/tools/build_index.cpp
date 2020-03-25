#include <spdlog/sinks/stdout_color_sinks.h>
#include <spdlog/spdlog.h>

#include <nlohmann/json.hpp>

int main(int argc, char const *argv[])
{
    spdlog::drop("");
    spdlog::set_default_logger(spdlog::stderr_color_mt(""));

    nlohmann::json do_nothing;
}