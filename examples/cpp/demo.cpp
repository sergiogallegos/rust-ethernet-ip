#include "eip_client.hpp"

#include <cmath>
#include <iostream>

using rust_ethernet_ip::EipClient;
using rust_ethernet_ip::require;
using rust_ethernet_ip::require_ok;
using rust_ethernet_ip::require_value;

std::uint64_t schema_generation(const std::string &json)
{
    const std::string marker = "\"generation\":";
    const auto start = json.find(marker);
    require(start != std::string::npos, "diagnostics missing schema generation");
    return std::stoull(json.substr(start + marker.size()));
}

int main(int argc, char **argv)
{
    if (argc != 2) {
        std::cerr << "usage: cpp_smoke_demo HOST:PORT\n";
        return 2;
    }

    auto client_result = EipClient::connect(argv[1]);
    if (!client_result) {
        std::cerr << "connect failed: " << client_result.error.message << "\n";
        return 1;
    }
    EipClient client = std::move(client_result.value);

    const auto schema_before = schema_generation(
        require_value(client.diagnosticsJson(), "diagnostics before schema refresh"));
    require_ok(client.refreshSchema(), "refresh controller schema");
    const auto schema_after = schema_generation(
        require_value(client.diagnosticsJson(), "diagnostics after schema refresh"));
    require(schema_after == schema_before + 1, "schema generation did not advance");

    require_ok(client.write_dint("DINT_TAG", 4242), "write DINT_TAG");
    require(require_value(client.read_dint("DINT_TAG"), "read DINT_TAG") == 4242, "DINT round trip mismatch");

    require_ok(client.write_real("REAL_TAG", 12.5), "write REAL_TAG");
    require(std::fabs(require_value(client.read_real("REAL_TAG"), "read REAL_TAG") - 12.5) < 0.001, "REAL round trip mismatch");

    require_ok(client.write_string("STRING_TAG", "CppSmoke"), "write STRING_TAG");
    require(require_value(client.read_string("STRING_TAG"), "read STRING_TAG") == "CppSmoke", "STRING round trip mismatch");

    const std::string write_payload =
        R"([{"tag_name":"DINT_TAG","value_type":"DINT","value":5001},{"tag_name":"REAL_TAG","value_type":"REAL","value":7.25},{"tag_name":"STRING_TAG","value_type":"STRING","value":"BatchCpp"}])";
    const auto write_result = require_value(client.write_tags_batch(write_payload, 3), "batch write");
    require(write_result.find("\"success\":true") != std::string::npos, "batch write did not report success");

    const auto read_result = require_value(
        client.read_tags_batch({ "DINT_TAG", "REAL_TAG", "STRING_TAG" }),
        "batch read");
    require(read_result.find("\"Dint\":5001") != std::string::npos, "batch read missing DINT value");
    require(read_result.find("BatchCpp") != std::string::npos, "batch read missing STRING value");

    std::cout << "C++ smoke passed\n";
    return 0;
}
