CREATE MIGRATION m1svpfvypikjtasavcqtqoxzgvs742xn2kzq2s3cormplilc7a7onq
    ONTO initial
{
  CREATE FUTURE nonrecursive_access_policies;
  CREATE TYPE default::Person {
      CREATE REQUIRED PROPERTY display_name -> std::str;
  };
};
