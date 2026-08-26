#pragma once

#include "types.hpp"

#include <QFuture>
#include <QString>
#include <QJsonValue>
#include <QJsonObject>
#include <QJsonArray>
#include <QJsonDocument>

#include <memory>

namespace #root_namespace {

template<typename T>
struct [[nodiscard]] Result {
    T result{};
    QString error_message{};
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
    QString error_message{};
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
    QJsonValue error_message = err["message"];
    int error_code = err["code"].toInt();
    if (!error_message.isString() || error_code == 0)
      return {{}, "Invalid error in response: " + QJsonDocument(val).toJson(QJsonDocument::Compact), -32700};
    return {{}, error_message.toString(), error_code};
  }
  if (!val.contains("result"))
    return {{}, "Neither error nor result in response: " + QJsonDocument(val).toJson(QJsonDocument::Compact), -32700};
  return {val["result"], {}, 0};
}

class Transport {
public:
  using CompletionHandler = std::function<void(const Result<QJsonValue>)>;
  virtual void send(const QString method, const QJsonValue request, CompletionHandler onCompleted) = 0;
};

class RawClient {
  std::unique_ptr<Transport> transport_;

  template <typename T>
  QFuture<Result<T>> request(const QString method, const QJsonArray params) {
    QFutureInterface<Result<T>> interface;
    interface.reportStarted();
    transport_->send(method, params,
        [method, interface](const Result<QJsonValue> val) mutable {
          interface.reportResult(mapToConcreteType<T>(val, method));
          interface.reportFinished();
        });
    return interface.future();
  }

  template <typename T>
  static Result<T> mapToConcreteType(const Result<QJsonValue> val, const QString method){
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
  }
public:
  RawClient(std::unique_ptr<Transport> t) : transport_{std::move(t)} {}

#methods
};

}
