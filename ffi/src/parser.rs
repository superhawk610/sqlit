// SQL statement grammar (abbrev.)
//
// https://www.sqlite.org/lang_select.html
//         select :  SELECT [ ALL | DISTINCT ] ( * | ( <ident> ... ) ) FROM <ident> ( ; )?
//
//       alphanum :  ( A-Z | a-z | 0-9 | _ )+
//
//          ident :  <alphanum>
//                   " <alphanum> "
//
//          value :  ( 0-9 )+
//                   ( 0-9 )+ . ( 0-9 )+
//                   ' ^' '
//
//     column-def :  <ident> <type> ( <col-constraint> )?+
//
// https://www.sqlite.org/datatype3.html
//           type :  INTEGER
//                   REAL
//                   TEXT
//                   BLOB
//
// col-constraint :  PRIMARY KEY ( AUTOINCREMENT )?
//                   DEFAULT <value>
//                   NOT NULL
//                   UNIQUE
//
//         filter :  ( WHERE | AND | OR ) <ident> <comparison> ...
//
//     comparison :  = <value>
//                   != <value>
//                   <> <value>
//                   > <value>
//                   >= <value>
//                   < <value>
//                   <= <value>
//                   LIKE <value>
//                   ILIKE <value>
//                   NOT LIKE <value>
//                   IS NULL
//                   ( IS )? NOT NULL
//
// https://www.sqlite.org/lang_insert.html
//         insert :  INSERT INTO <ident> \( <field> \) VALUES ( \( <value> ... \) ... ) ( ; )?
//
//         update :  UPDATE <ident> SET ( <assignment> )+ ( filter )? ( ; )?
//
//         delete :  DELETE FROM <ident> filter ( ; )?
//
// https://www.sqlite.org/lang_createtable.html
//         create :  CREATE TABLE ( IF NOT EXISTS )? <ident> \( <column-def> ... \) ( ; )?
//
// https://www.sqlite.org/lang_droptable.html
//           drop :  DROP TABLE ( IF EXISTS )? <ident> ( ; )?
