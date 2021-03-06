#include "DeclareCommand.hpp"
#include "serialization.hpp"
#include "RukoDb.hpp"
#include "parsing.hpp"

DeclareCommand::DeclareCommand(const Byte *data, size_t &p) :
        keys(deserialize<Vec<Str>>(data, p)),
        dataType(deserialize<Byte>(data, p)),
        indices(deserialize<Vec<Str>>(data, p)) {
    if (dataType == 0) {
        throw std::invalid_argument("Invalid data type");
    }
}

DeclareCommand::DeclareCommand(const char *&data) :
        keys(parseLocation(data)),
        dataType(Byte(parseInt(data))),
        indices(*data ? parseList(data) : Vec<Str>()) {}

Str DeclareCommand::toString() {
    return "DECLARE " + join(keys, ".") + " " + std::to_string(dataType) +
           (indices.empty() ? "" : " " + join(indices, ","));
}

CommandResult DeclareCommand::perform(RukoDb &db) {
    bool write = db.declare(keys, dataType, indices);
    return {{}, write};
}
