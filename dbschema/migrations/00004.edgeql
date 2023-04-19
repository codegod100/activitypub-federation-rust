CREATE MIGRATION m1bn3mgaaqw2yajz2wngqsr3ecew4q47n2nr2xgb2lebdp3heibpma
    ONTO m1ediq7dx5uhlahzgdqxdeioawk5haa42kdvlkrhpdclzsl4gwqrjq
{
  ALTER TYPE default::Person {
      DROP PROPERTY attributes;
  };
  ALTER TYPE default::Person {
      CREATE MULTI PROPERTY links -> tuple<rel: std::str, type: std::str, href: std::str>;
  };
};
