#pragma once

#include "types.hpp"

#include <QString>
#include <QJsonValue>
#include <QJsonObject>
#include <QJsonArray>
#include <QJsonDocument>

#include <future>

template<typename T>
struct Result {
    T result;
    QString error_message;
    int32_t error_code = 0;
};

template<>
struct Result<void> {
    QString error_message;
    int32_t error_code = 0;
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
    Transport* transport_;

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
    RawClient(Transport* transport)
        : transport_(transport) {}


  std::future<Result<QString>> shoutAsync(QString msg) {
    return request<QString>("shout", QJsonArray{toJson(msg)});
  }

  QString shout(QString msg) {
    return request<QString>("shout", QJsonArray{toJson(msg)}).get().result;
  }


  std::future<Result<float>> addAsync(float a, float b) {
    return request<float>("add", QJsonArray{toJson(a), toJson(b)});
  }

  float add(float a, float b) {
    return request<float>("add", QJsonArray{toJson(a), toJson(b)}).get().result;
  }


};
