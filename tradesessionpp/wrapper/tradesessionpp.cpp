#include <iostream>

#include "tradesessionpp.h"

using namespace std;

namespace sessionrs {
static ::rust::Box<::sessionpp::SessionPP>
from_minutes(const std::vector<::uint16_t> &minutes) {
  ::rust::Vec<::uint16_t> vec;
  vec.reserve(minutes.size());
  for (auto &m : minutes)
    vec.emplace_back(m);
  auto box = sessionpp::new_from_minutes(std::move(vec));
  return box;
}

tradesession::tradesession(const std::vector<::uint16_t> &minutes)
    : box_(from_minutes(minutes)) {}

vector<::uint16_t> tradesession::minutes_list() const {
  ::rust::Vec<::uint16_t> vec = box_->minutes_list();
  return vector<::uint16_t>(vec.begin(), vec.end());
}

bool tradesession::add_slice(unsigned start_hour, unsigned start_minute,
                             unsigned end_hour, unsigned end_minute,
                             string &error) {
  try {
    box_->add_slice(start_hour, start_minute, end_hour, end_minute);
    return true;
  } catch (const exception &e) {
    error = "tradesession: add_slice failed, " + string(e.what());
    std::cerr << error << endl;
    return false;
  }
}

//////////////////////////////////////////////////////////////////////////

optional<tradesession> session_mgr::get_session(const string &product) {
  try {
    auto session = box_->get_session(product);
    return make_optional<tradesession>(std::move(session));
  } catch (const exception &e) {
    cerr << "session_mgr: error getting session for product '" << product
         << "': " << e.what() << endl;
    return nullopt;
  }
};

bool session_mgr::load_from_csv(const string &csv_file_path, string &error,
                                bool merge) {
  try {
    box_->reload_csv_file(csv_file_path, merge);
    return true;
  } catch (const exception &e) {
    error = "session_mgr: failed to load session from CSV: " + string(e.what());
    cerr << error << endl;
    return false;
  }
}

bool session_mgr::load_from_csv_content(const string &csv_content,
                                        string &error, bool merge) {
  try {
    box_->reload_csv_content(csv_content, merge);
    return true;
  } catch (const exception &e) {
    error = "session_mgr: failed to load session from CSV content: " +
            string(e.what());
    cerr << error << endl;
    return false;
  }
}

} // namespace sessionrs