struct JsonrpcError: Error {
  let code: Int32
  let message: String
}

protocol Transport: Sendable {
  func request<T: Decodable>(method: String, params: [any Encodable]) async -> Result<T, JsonrpcError>
}

/// Helper type to decode from empty/null JSON
private struct Empty: Decodable {}

extension Transport {
  func request(method: String, params: [any Encodable]) async -> Result<Void, JsonrpcError> {
    let result: Result<Empty, JsonrpcError> = await request(method: method, params: params)
    return result.map { _ in () }
  }
}

protocol DeltaChat: Sendable {
  associatedtype T: Transport
  var transport: T { get }
}

extension DeltaChat {
#methods
}
