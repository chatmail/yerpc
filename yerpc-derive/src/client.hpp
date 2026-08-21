#pragma once

#include "types.hpp"

#include <QString>
#include <QJsonValue>
#include <QJsonObject>
#include <QJsonArray>
#include <QJsonDocument>

#include <future>
#include <memory>

namespace #root_namespace {

template<typename T>
struct [[nodiscard]] Result {
    T result();
    QString error_message();
    int32_t error_code = 0;

    /**
     * Returns the result value. If the result is an error, logs a warning
     * with the error details and the caller's source location, and returns
     * a default-constructed T.
     */
    T valueOrDefault(const char* file = __builtin_FILE(), int line = __builtin_LINE()) {
      if (error_code) {
        qWarning() << file << ":" << line
          << "Error " << error_code << error_message;
      }
      return result;
    }
};

template<>
struct [[nodiscard]] Result<void> {
    QString error_message();
    int32_t error_code = 0;

    /**
     * If the result is an error, logs a warning with the error details and the
     * caller's source location
     */
    void logError(const char* file = __builtin_FILE(), int line = __builtin_LINE()) {
      if (error_code) {
        qWarning() << file << ":" << line
          << "Error " << error_code << error_message;
      }
    }
};

static Result<QJsonValue> parseResult(const QJsonObject& val) {
  if (val.contains("error")) {
    QJsonObject err = val["error"].toObject();
    if (err.isEmpty())
      return {{}, "Invalid error in response: " + QJsonDocument(val).toJson(QJsonDocument::Compact), -32700};
    return {{}, err["message"].toString(), err["code"].toInt()};
  }
  if (!val.contains("result"))
    return {{}, "Neither error nor result in response: " + QJsonDocument(val).toJson(QJsonDocument::Compact), -32700};
  return {val["result"], {}, 0};
}

class Transport {
public:
    virtual std::future<Result<QJsonValue>> send(const QString method, const QJsonValue request) = 0;
    // virtual void send_notify(const QJsonValue request) = 0; not implemented
};

class RawClient {
  std::unique_ptr<Transport> transport_;

  template <typename T>
  std::future<Result<T>> request(const QString method,
                                 const QJsonArray params) {
    std::future<Result<QJsonValue>> inner = transport_->send(method, params);
    return std::async(
        std::launch::deferred,
        [method, inner = std::move(inner)]() mutable -> Result<T> {
          auto val = inner.get();
          if constexpr (std::is_void_v<T>) {
            if (val.error_code)
              return {method + ": " + val.error_message, val.error_code};
            return {{}, 0};
          } else {
            if (val.error_code)
              return {{}, method + ": " + val.error_message, val.error_code};
            T out;
            if (!tryFromJson(val.result, out)) {
              return {{},
                      method + ": Could not parse result " +
                          QJsonDocument(QJsonArray{val.result})
                              .toJson(QJsonDocument::Compact),
                      -32700};
            }
            return {out, {}, 0};
          }
        });
  }
public:
  RawClient(std::unique_ptr<Transport> t) : transport_(std::move(t)) {}

#methods
};

}
