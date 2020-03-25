#include <spdlog/sinks/stdout_color_sinks.h>
#include <spdlog/spdlog.h>

int main(int argc, char const *argv[])
{
    spdlog::drop("");
    spdlog::set_default_logger(spdlog::stderr_color_mt(""));

}