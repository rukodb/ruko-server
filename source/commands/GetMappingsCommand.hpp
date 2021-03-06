#pragma once

#include "Command.hpp"

class GetMappingsCommand : public Command {
public:
    GetMappingsCommand(const Byte *data, size_t &p);
    GetMappingsCommand(const char *&data);
    Str toString() override;
    CommandResult perform(RukoDb &db) override;

private:
    Vec<Str> location;
};
