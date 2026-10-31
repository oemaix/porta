#include "AccessPointTry.h"

#include <cstdio>
#include <cstdlib>
#include <stdexcept>
#include <string>
#include <vector>

static int failures = 0;

static void expect(bool cond, const char* what) {
  if (!cond) {
    failures++;
    // A failing check stays visible when the test binary is run by itself.
    fprintf(stderr, "fail: %s\n", what);
  }
}

int main() {
  std::vector<std::string> tried;
  cspot::tryAccessPoints(
      {"ap:4070", "ap:443", "ap:80"}, [&](const std::string& address) {
        tried.push_back(address);
        if (address == "ap:4070") {
          throw std::runtime_error("Error in read");
        }
      });
  expect(tried.size() == 2, "stops after the second address");
  expect(tried.size() > 0 && tried[0] == "ap:4070", "tries 4070 first");
  expect(tried.size() > 1 && tried[1] == "ap:443", "tries 443 after 4070 throws");

  tried.clear();
  bool threw = false;
  try {
    cspot::tryAccessPoints({"ap:4070", "ap:443"}, [&](const std::string& address) {
      tried.push_back(address);
      throw std::runtime_error(address);
    });
  } catch (const std::runtime_error& ex) {
    threw = std::string(ex.what()) == "ap:443";
  }
  expect(threw, "the last failure propagates");
  expect(tried.size() == 2, "both addresses were tried");

  threw = false;
  try {
    cspot::tryAccessPoints({}, [&](const std::string&) {});
  } catch (const std::runtime_error& ex) {
    threw = std::string(ex.what()) == "no access point";
  }
  expect(threw, "an empty list throws");

  return failures == 0 ? EXIT_SUCCESS : EXIT_FAILURE;
}
