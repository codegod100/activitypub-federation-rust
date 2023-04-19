CREATE MIGRATION m1fup2neja4xrbxdbrrsobvm6hgr5q7ukfidb2ys7i3xuavub3oakq
    ONTO m1bn3mgaaqw2yajz2wngqsr3ecew4q47n2nr2xgb2lebdp3heibpma
{
  ALTER TYPE default::Person {
      ALTER PROPERTY display_name {
          RENAME TO username;
      };
  };
};
