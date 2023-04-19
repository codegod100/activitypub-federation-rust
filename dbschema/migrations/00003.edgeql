CREATE MIGRATION m1ediq7dx5uhlahzgdqxdeioawk5haa42kdvlkrhpdclzsl4gwqrjq
    ONTO m1hytpdejm66jmobszmmblpnx63tgh376epqvyttonj2atyrvyea7a
{
  ALTER TYPE default::Person {
      CREATE MULTI PROPERTY attributes -> tuple<std::str, std::str>;
  };
};
