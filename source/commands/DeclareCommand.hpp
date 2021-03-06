#pragma once

#include "Command.hpp"

class DeclareCommand : public Command {
public:
    DeclareCommand(const Byte *data, size_t &p);
    DeclareCommand(const char *&data);
    Str toString() override;
    CommandResult perform(RukoDb &db) override;

private:
    Vec<Str> keys;
    Byte dataType;
    Vec<Str> indices;
};
