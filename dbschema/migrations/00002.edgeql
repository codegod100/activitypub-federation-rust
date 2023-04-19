CREATE MIGRATION m1hytpdejm66jmobszmmblpnx63tgh376epqvyttonj2atyrvyea7a
    ONTO m1svpfvypikjtasavcqtqoxzgvs742xn2kzq2s3cormplilc7a7onq
{
  ALTER TYPE default::Person {
      ALTER PROPERTY display_name {
          CREATE CONSTRAINT std::exclusive;
      };
  };
};
