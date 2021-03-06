#include "IndexNode.hpp"
#include "serialization.hpp"
#include "objects/Object.hpp"


IndexNode::IndexNode(const Byte *data, size_t &p) :
        children(deserialize<Map<Str, IndexNode::Ptr>>(data, p)),
        mapper(deserializePtr<KeyMapper>(data, p)) {}


Bytes IndexNode::toBytes() const {
    return concat(serialize(children), serializePtr(mapper));
}

KeyMapper &IndexNode::getMapper() {
    if (!mapper) {
        mapper = std::make_unique<KeyMapper>();
    }
    return *mapper;
}
