#pragma once
#include <chrono>
#include <optional>
#include <string>
#include <vector>

#if defined(_WIN64) || defined(_WIN32)
#ifdef SESSIONRS_EXPORTS
#define SESSIONRS_API __declspec(dllexport)
#else
#define SESSIONRS_API __declspec(dllimport)
#endif
#else
#define SESSIONRS_API
#endif

#ifdef _MSC_VER
#pragma warning(push)
#pragma warning(disable : 4251) // 需要有 dll 接口...
#endif

#include "tradesessionpp.rs.h"

namespace sessionrs {

using namespace std;

class SESSIONRS_API tradesession {
private:
  ::rust::Box<sessionpp::SessionPP> box_;

public:
  static tradesession new_full_session() noexcept {
    return tradesession(sessionpp::new_full_session());
  }
  static tradesession new_stock_session() noexcept {
    return tradesession(sessionpp::new_stock_session());
  }
  static tradesession new_stock_index_session() noexcept {
    return tradesession(sessionpp::new_stock_index_session());
  }
  static tradesession new_commodity_session() noexcept {
    return tradesession(sessionpp::new_commodity_session());
  }
  static tradesession new_commodity_session_night() noexcept {
    return tradesession(sessionpp::new_commodity_session_night());
  }
  static tradesession new_bond_session() noexcept {
    return tradesession(sessionpp::new_bond_session());
  }

public:
  tradesession() : box_(sessionpp::new_session()) {};
  tradesession(::rust::Box<sessionpp::SessionPP> box) : box_(std::move(box)) {};
  // 注意：minutes里面的时间是增加4小时之后的
  tradesession(const vector<std::uint16_t> &minutes);

  vector<::uint16_t> minutes_list() const;

  string to_string() const { return (string)box_->to_string(); }

  bool add_slice(unsigned start_hour, unsigned start_minute, unsigned end_hour,
                 unsigned end_minute, string &error);

  void post_fix() noexcept { box_->post_fix(); }

  // 该品种日线开始时间，9:00/9:15/9:30/21:00, 一般是集合竞价所在的时间
  template <typename _Duration = chrono::seconds>
  _Duration day_begin() const noexcept {
    return chrono::duration_cast<_Duration>(chrono::nanoseconds(_day_begin()));
  }

  // 该品种早盘开始时间，9:00/9:15/9:30,非夜盘品种跟day_begin相同
  template <typename _Duration = chrono::seconds>
  _Duration morning_begin() const noexcept {
    return chrono::duration_cast<_Duration>(
        chrono::nanoseconds(_morning_begin()));
  }

  // 该品种日线结束时间，商品15:00，股指曾经15:15，股指现在15:00
  template <typename _Duration = chrono::seconds>
  _Duration day_end() const noexcept {
    return chrono::duration_cast<_Duration>(chrono::nanoseconds(_day_end()));
  }

  template <typename _Duration = chrono::seconds>
  bool in_session(_Duration time, bool include_begin = true,
                  bool include_end = false) const noexcept {
    auto nanos_since_midnight =
        chrono::duration_cast<chrono::nanoseconds>(time);
    return _in_session(nanos_since_midnight.count(), include_begin,
                       include_end);
  }
  template <typename _Duration = chrono::seconds>
  bool any_in_session(_Duration start, _Duration end,
                      bool include_begin_end) const noexcept {
    auto start_nanos = chrono::duration_cast<chrono::nanoseconds>(start);
    auto end_nanos = chrono::duration_cast<chrono::nanoseconds>(end);
    return _any_in_session(start_nanos.count(), end_nanos.count(),
                           include_begin_end);
  }

protected:
  // 避免template暴露box_对象,
  // tradessionpp.rs.h里面的方法在windows下面没有dllexport

  int64_t _day_begin() const noexcept { return box_->day_begin(); }
  int64_t _morning_begin() const noexcept { return box_->morning_begin(); }
  int64_t _day_end() const noexcept { return box_->day_end(); }
  bool _in_session(int64_t nanos_since_midnight, bool include_begin,
                   bool include_end) const noexcept {
    return box_->in_session(nanos_since_midnight, include_begin, include_end);
  };

  bool _any_in_session(int64_t start, int64_t end,
                       bool include_begin_end) const noexcept {
    return box_->any_in_session(start, end, include_begin_end);
  }
};

class SESSIONRS_API session_mgr {
private:
  ::rust::Box<sessionpp::SessionMgr> box_;

public:
  session_mgr() : box_(sessionpp::new_mgr()) {};

  optional<tradesession> get_session(const string &product);

  bool load_from_csv(const string &csv_file_path, string &error,
                     bool merge = true);

  bool load_from_csv_content(const string &csv_content, string &error,
                             bool merge = true);

  size_t sessions_count() const { return box_->sessions_count(); }

  // 这里没有做转换，直接返回原始的::rust类型
  ::rust::Vec<::rust::String> session_map_keys() {
    return box_->session_map_keys();
  }
};

} // namespace sessionrs

#ifdef _MSC_VER
#pragma warning(pop)
#endif