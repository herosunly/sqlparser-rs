extern crate log;
extern crate sqlparser;

use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::sqlast::*;
use sqlparser::sqlparser::*;
use sqlparser::sqltokenizer::*;

use log::*;

#[test]
fn test_prev_index() {
    let sql: &str = "SELECT version()";
    let mut parser = parser(sql);
    assert_eq!(parser.prev_token(), None);
    assert_eq!(parser.next_token(), Some(Token::Keyword("SELECT".into())));
    assert_eq!(
        parser.next_token(),
        Some(Token::Identifier("version".into()))
    );
    assert_eq!(
        parser.prev_token(),
        Some(Token::Identifier("version".into()))
    );
    assert_eq!(
        parser.peek_token(),
        Some(Token::Identifier("version".into()))
    );
    assert_eq!(parser.prev_token(), Some(Token::Keyword("SELECT".into())));
    assert_eq!(parser.prev_token(), None);
}

#[test]
fn parse_delete_statement() {
    let sql: &str = "DELETE FROM 'table'";
    let ast = parse_sql(sql);
    assert_eq!("DELETE FROM 'table'", ast.to_string());

    match parse_sql(&sql) {
        ASTNode::SQLDelete { relation, .. } => {
            assert_eq!(
                Some(Box::new(ASTNode::SQLValue(Value::SingleQuotedString(
                    "table".to_string()
                )))),
                relation
            );
        }

        _ => assert!(false),
    }
}

#[test]
fn parse_where_delete_statement() {
    let sql: &str = "DELETE FROM 'table' WHERE name = 5";
    let ast = parse_sql(sql);
    println!("ast: {:#?}", ast);
    assert_eq!(sql, ast.to_string());

    use self::ASTNode::*;
    use self::SQLOperator::*;

    match parse_sql(&sql) {
        ASTNode::SQLDelete {
            relation,
            selection,
            ..
        } => {
            assert_eq!(
                Some(Box::new(ASTNode::SQLValue(Value::SingleQuotedString(
                    "table".to_string()
                )))),
                relation
            );

            assert_eq!(
                SQLBinaryExpr {
                    left: Box::new(SQLIdentifier("name".to_string())),
                    op: Eq,
                    right: Box::new(ASTNode::SQLValue(Value::Long(5))),
                },
                *selection.unwrap(),
            );
        }

        _ => assert!(false),
    }
}

#[test]
fn parse_simple_select() {
    let sql = String::from("SELECT id, fname, lname FROM customer WHERE id = 1 LIMIT 5");
    let ast = parse_sql(&sql);
    assert_eq!(sql, ast.to_string());
    match ast {
        ASTNode::SQLSelect {
            projection, limit, ..
        } => {
            assert_eq!(3, projection.len());
            assert_eq!(Some(Box::new(ASTNode::SQLValue(Value::Long(5)))), limit);
        }
        _ => assert!(false),
    }
}

#[test]
fn parse_simple_insert() {
    let sql = String::from("INSERT INTO customer VALUES(1, 2, 3)");
    let ast = parse_sql(&sql);
    assert_eq!(sql, ast.to_string());
    match ast {
        ASTNode::SQLInsert {
            table_name,
            columns,
            values,
            ..
        } => {
            assert_eq!(table_name, "customer");
            assert!(columns.is_empty());
            assert_eq!(
                vec![vec![
                    ASTNode::SQLValue(Value::Long(1)),
                    ASTNode::SQLValue(Value::Long(2)),
                    ASTNode::SQLValue(Value::Long(3))
                ]],
                values
            );
        }
        _ => assert!(false),
    }
}

#[test]
fn parse_common_insert() {
    let sql = String::from("INSERT INTO public.customer VALUES(1, 2, 3)");
    let ast = parse_sql(&sql);
    assert_eq!(sql, ast.to_string());
    match ast {
        ASTNode::SQLInsert {
            table_name,
            columns,
            values,
            ..
        } => {
            assert_eq!(table_name, "public.customer");
            assert!(columns.is_empty());
            assert_eq!(
                vec![vec![
                    ASTNode::SQLValue(Value::Long(1)),
                    ASTNode::SQLValue(Value::Long(2)),
                    ASTNode::SQLValue(Value::Long(3))
                ]],
                values
            );
        }
        _ => assert!(false),
    }
}

#[test]
fn parse_complex_insert() {
    let sql = String::from("INSERT INTO db.public.customer VALUES(1, 2, 3)");
    let ast = parse_sql(&sql);
    assert_eq!(sql, ast.to_string());
    match ast {
        ASTNode::SQLInsert {
            table_name,
            columns,
            values,
            ..
        } => {
            assert_eq!(table_name, "db.public.customer");
            assert!(columns.is_empty());
            assert_eq!(
                vec![vec![
                    ASTNode::SQLValue(Value::Long(1)),
                    ASTNode::SQLValue(Value::Long(2)),
                    ASTNode::SQLValue(Value::Long(3))
                ]],
                values
            );
        }
        _ => assert!(false),
    }
}

#[test]
fn parse_invalid_table_name() {
    let mut parser = parser("db.public..customer");
    let ast = parser.parse_tablename();
    assert!(ast.is_err());
}

#[test]
fn parse_insert_with_columns() {
    let sql = String::from("INSERT INTO public.customer (id, name, active) VALUES(1, 2, 3)");
    let ast = parse_sql(&sql);
    assert_eq!(sql, ast.to_string());
    match ast {
        ASTNode::SQLInsert {
            table_name,
            columns,
            values,
            ..
        } => {
            assert_eq!(table_name, "public.customer");
            assert_eq!(
                columns,
                vec!["id".to_string(), "name".to_string(), "active".to_string()]
            );
            assert_eq!(
                vec![vec![
                    ASTNode::SQLValue(Value::Long(1)),
                    ASTNode::SQLValue(Value::Long(2)),
                    ASTNode::SQLValue(Value::Long(3))
                ]],
                values
            );
        }
        _ => assert!(false),
    }
}

#[test]
fn parse_insert_invalid() {
    let sql = String::from("INSERT public.customer (id, name, active) VALUES (1, 2, 3)");
    let mut parser = parser(&sql);
    match parser.parse() {
        Err(_) => {}
        _ => assert!(false),
    }
}

#[test]
fn parse_select_wildcard() {
    let sql = String::from("SELECT * FROM customer");
    let ast = parse_sql(&sql);
    assert_eq!(sql, ast.to_string());
    match ast {
        ASTNode::SQLSelect { projection, .. } => {
            assert_eq!(1, projection.len());
            assert_eq!(ASTNode::SQLWildcard, projection[0]);
        }
        _ => assert!(false),
    }
}

#[test]
fn parse_select_count_wildcard() {
    let sql = String::from("SELECT COUNT(*) FROM customer");
    let ast = parse_sql(&sql);
    assert_eq!(sql, ast.to_string());
    match ast {
        ASTNode::SQLSelect { projection, .. } => {
            assert_eq!(1, projection.len());
            assert_eq!(
                ASTNode::SQLFunction {
                    id: "COUNT".to_string(),
                    args: vec![ASTNode::SQLWildcard],
                },
                projection[0]
            );
        }
        _ => assert!(false),
    }
}

#[test]
fn parse_select_string_predicate() {
    let sql = String::from(
        "SELECT id, fname, lname FROM customer \
         WHERE salary != 'Not Provided' AND salary != ''",
    );
    let ast = parse_sql(&sql);
    assert_eq!(sql, ast.to_string());
}

#[test]
fn parse_projection_nested_type() {
    let sql = String::from("SELECT customer.address.state FROM foo");
    let ast = parse_sql(&sql);
    assert_eq!(sql, ast.to_string());
}

#[test]
fn parse_compound_expr_1() {
    use self::ASTNode::*;
    use self::SQLOperator::*;
    let sql = String::from("a + b * c");
    let ast = parse_sql(&sql);
    assert_eq!(sql, ast.to_string());
    assert_eq!(
        SQLBinaryExpr {
            left: Box::new(SQLIdentifier("a".to_string())),
            op: Plus,
            right: Box::new(SQLBinaryExpr {
                left: Box::new(SQLIdentifier("b".to_string())),
                op: Multiply,
                right: Box::new(SQLIdentifier("c".to_string()))
            })
        },
        ast
    );
}

#[test]
fn parse_compound_expr_2() {
    use self::ASTNode::*;
    use self::SQLOperator::*;
    let sql = String::from("a * b + c");
    let ast = parse_sql(&sql);
    assert_eq!(sql, ast.to_string());
    assert_eq!(
        SQLBinaryExpr {
            left: Box::new(SQLBinaryExpr {
                left: Box::new(SQLIdentifier("a".to_string())),
                op: Multiply,
                right: Box::new(SQLIdentifier("b".to_string()))
            }),
            op: Plus,
            right: Box::new(SQLIdentifier("c".to_string()))
        },
        ast
    );
}

#[test]
fn parse_is_null() {
    use self::ASTNode::*;
    let sql = String::from("a IS NULL");
    let ast = parse_sql(&sql);
    assert_eq!(sql, ast.to_string());
    assert_eq!(SQLIsNull(Box::new(SQLIdentifier("a".to_string()))), ast);
}

#[test]
fn parse_is_not_null() {
    use self::ASTNode::*;
    let sql = String::from("a IS NOT NULL");
    let ast = parse_sql(&sql);
    assert_eq!(sql, ast.to_string());
    assert_eq!(SQLIsNotNull(Box::new(SQLIdentifier("a".to_string()))), ast);
}

#[test]
fn parse_select_order_by() {
    let sql = String::from(
        "SELECT id, fname, lname FROM customer WHERE id < 5 ORDER BY lname ASC, fname DESC",
    );
    let ast = parse_sql(&sql);
    assert_eq!(sql, ast.to_string());
    match ast {
        ASTNode::SQLSelect { order_by, .. } => {
            assert_eq!(
                Some(vec![
                    SQLOrderByExpr {
                        expr: Box::new(ASTNode::SQLIdentifier("lname".to_string())),
                        asc: true,
                    },
                    SQLOrderByExpr {
                        expr: Box::new(ASTNode::SQLIdentifier("fname".to_string())),
                        asc: false,
                    },
                ]),
                order_by
            );
        }
        _ => assert!(false),
    }
}

#[test]
fn parse_select_group_by() {
    let sql = String::from("SELECT id, fname, lname FROM customer GROUP BY lname, fname");
    let ast = parse_sql(&sql);
    assert_eq!(sql, ast.to_string());
    match ast {
        ASTNode::SQLSelect { group_by, .. } => {
            assert_eq!(
                Some(vec![
                    ASTNode::SQLIdentifier("lname".to_string()),
                    ASTNode::SQLIdentifier("fname".to_string()),
                ]),
                group_by
            );
        }
        _ => assert!(false),
    }
}

#[test]
fn parse_limit_accepts_all() {
    let sql = String::from("SELECT id, fname, lname FROM customer WHERE id = 1 LIMIT ALL");
    let expected = String::from("SELECT id, fname, lname FROM customer WHERE id = 1");
    let ast = parse_sql(&sql);
    assert_eq!(expected, ast.to_string());
    match ast {
        ASTNode::SQLSelect {
            projection, limit, ..
        } => {
            assert_eq!(3, projection.len());
            assert_eq!(None, limit);
        }
        _ => assert!(false),
    }
}

#[test]
fn parse_cast() {
    let sql = String::from("SELECT CAST(id AS bigint) FROM customer");
    let ast = parse_sql(&sql);
    assert_eq!(sql, ast.to_string());
    match ast {
        ASTNode::SQLSelect { projection, .. } => {
            assert_eq!(1, projection.len());
            assert_eq!(
                ASTNode::SQLCast {
                    expr: Box::new(ASTNode::SQLIdentifier("id".to_string())),
                    data_type: SQLType::BigInt
                },
                projection[0]
            );
        }
        _ => assert!(false),
    }
}

#[test]
fn parse_create_table() {
    let sql = String::from(
        "CREATE TABLE uk_cities (\
         name VARCHAR(100) NOT NULL,\
         lat DOUBLE NULL,\
         lng DOUBLE NULL)",
    );
    let expected = String::from(
        "CREATE TABLE uk_cities (\
         name character varying(100) NOT NULL, \
         lat double, \
         lng double)",
    );
    let ast = parse_sql(&sql);
    assert_eq!(expected, ast.to_string());
    match ast {
        ASTNode::SQLCreateTable { name, columns } => {
            assert_eq!("uk_cities", name);
            assert_eq!(3, columns.len());

            let c_name = &columns[0];
            assert_eq!("name", c_name.name);
            assert_eq!(SQLType::Varchar(Some(100)), c_name.data_type);
            assert_eq!(false, c_name.allow_null);

            let c_lat = &columns[1];
            assert_eq!("lat", c_lat.name);
            assert_eq!(SQLType::Double, c_lat.data_type);
            assert_eq!(true, c_lat.allow_null);

            let c_lng = &columns[2];
            assert_eq!("lng", c_lng.name);
            assert_eq!(SQLType::Double, c_lng.data_type);
            assert_eq!(true, c_lng.allow_null);
        }
        _ => assert!(false),
    }
}

#[test]
fn parse_create_table_with_defaults() {
    let sql = String::from(
        "CREATE TABLE public.customer (
            customer_id integer DEFAULT nextval(public.customer_customer_id_seq) NOT NULL,
            store_id smallint NOT NULL,
            first_name character varying(45) NOT NULL,
            last_name character varying(45) NOT NULL,
            email character varying(50),
            address_id smallint NOT NULL,
            activebool boolean DEFAULT true NOT NULL,
            create_date date DEFAULT now()::text NOT NULL,
            last_update timestamp without time zone DEFAULT now() NOT NULL,
            active integer NOT NULL)",
    );
    let ast = parse_sql(&sql);
    match ast {
        ASTNode::SQLCreateTable { name, columns } => {
            assert_eq!("public.customer", name);
            assert_eq!(10, columns.len());

            let c_name = &columns[0];
            assert_eq!("customer_id", c_name.name);
            assert_eq!(SQLType::Int, c_name.data_type);
            assert_eq!(false, c_name.allow_null);

            let c_lat = &columns[1];
            assert_eq!("store_id", c_lat.name);
            assert_eq!(SQLType::SmallInt, c_lat.data_type);
            assert_eq!(false, c_lat.allow_null);

            let c_lng = &columns[2];
            assert_eq!("first_name", c_lng.name);
            assert_eq!(SQLType::Varchar(Some(45)), c_lng.data_type);
            assert_eq!(false, c_lng.allow_null);
        }
        _ => assert!(false),
    }
}

#[test]
fn parse_create_table_from_pg_dump() {
    let sql = String::from("
        CREATE TABLE public.customer (
            customer_id integer DEFAULT nextval('public.customer_customer_id_seq'::regclass) NOT NULL,
            store_id smallint NOT NULL,
            first_name character varying(45) NOT NULL,
            last_name character varying(45) NOT NULL,
            info text[],
            address_id smallint NOT NULL,
            activebool boolean DEFAULT true NOT NULL,
            create_date date DEFAULT now()::date NOT NULL,
            create_date1 date DEFAULT 'now'::text::date NOT NULL,
            last_update timestamp without time zone DEFAULT now(),
            release_year public.year,
            active integer
        )");
    let ast = parse_sql(&sql);
    match ast {
        ASTNode::SQLCreateTable { name, columns } => {
            assert_eq!("public.customer", name);

            let c_customer_id = &columns[0];
            assert_eq!("customer_id", c_customer_id.name);
            assert_eq!(SQLType::Int, c_customer_id.data_type);
            assert_eq!(false, c_customer_id.allow_null);

            let c_store_id = &columns[1];
            assert_eq!("store_id", c_store_id.name);
            assert_eq!(SQLType::SmallInt, c_store_id.data_type);
            assert_eq!(false, c_store_id.allow_null);

            let c_first_name = &columns[2];
            assert_eq!("first_name", c_first_name.name);
            assert_eq!(SQLType::Varchar(Some(45)), c_first_name.data_type);
            assert_eq!(false, c_first_name.allow_null);

            let c_create_date1 = &columns[8];
            assert_eq!(
                Some(Box::new(ASTNode::SQLCast {
                    expr: Box::new(ASTNode::SQLCast {
                        expr: Box::new(ASTNode::SQLValue(Value::SingleQuotedString(
                            "now".to_string()
                        ))),
                        data_type: SQLType::Text
                    }),
                    data_type: SQLType::Date
                })),
                c_create_date1.default
            );

            let c_release_year = &columns[10];
            assert_eq!(
                SQLType::Custom("public.year".to_string()),
                c_release_year.data_type
            );
        }
        _ => assert!(false),
    }
}

#[test]
fn parse_create_table_with_inherit() {
    let sql = String::from(
        "\
         CREATE TABLE bazaar.settings (\
         settings_id uuid PRIMARY KEY DEFAULT uuid_generate_v4() NOT NULL, \
         user_id uuid UNIQUE, \
         value text[], \
         use_metric boolean DEFAULT true\
         )",
    );
    let ast = parse_sql(&sql);
    assert_eq!(sql, ast.to_string());
    match ast {
        ASTNode::SQLCreateTable { name, columns } => {
            assert_eq!("bazaar.settings", name);

            let c_name = &columns[0];
            assert_eq!("settings_id", c_name.name);
            assert_eq!(SQLType::Uuid, c_name.data_type);
            assert_eq!(false, c_name.allow_null);
            assert_eq!(true, c_name.is_primary);
            assert_eq!(false, c_name.is_unique);

            let c_name = &columns[1];
            assert_eq!("user_id", c_name.name);
            assert_eq!(SQLType::Uuid, c_name.data_type);
            assert_eq!(true, c_name.allow_null);
            assert_eq!(false, c_name.is_primary);
            assert_eq!(true, c_name.is_unique);
        }
        _ => assert!(false),
    }
}

#[test]
fn parse_alter_table_constraint_primary_key() {
    let sql = String::from(
        "\
         ALTER TABLE bazaar.address \
         ADD CONSTRAINT address_pkey PRIMARY KEY (address_id)",
    );
    let ast = parse_sql(&sql);
    println!("ast: {:?}", ast);
    assert_eq!(sql, ast.to_string());
    match ast {
        ASTNode::SQLAlterTable { name, .. } => {
            assert_eq!(name, "bazaar.address");
        }
        _ => assert!(false),
    }
}

#[test]
fn parse_alter_table_constraint_foreign_key() {
    let sql = String::from("\
    ALTER TABLE public.customer \
        ADD CONSTRAINT customer_address_id_fkey FOREIGN KEY (address_id) REFERENCES public.address(address_id)");
    let ast = parse_sql(&sql);
    assert_eq!(sql, ast.to_string());
    println!("ast: {:?}", ast);
    match ast {
        ASTNode::SQLAlterTable { name, .. } => {
            assert_eq!(name, "public.customer");
        }
        _ => assert!(false),
    }
}

#[test]
fn parse_copy_example() {
    let sql = String::from(r#"COPY public.actor (actor_id, first_name, last_name, last_update, value) FROM stdin;
1	PENELOPE	GUINESS	2006-02-15 09:34:33 0.11111
2	NICK	WAHLBERG	2006-02-15 09:34:33 0.22222
3	ED	CHASE	2006-02-15 09:34:33 0.312323
4	JENNIFER	DAVIS	2006-02-15 09:34:33 0.3232
5	JOHNNY	LOLLOBRIGIDA	2006-02-15 09:34:33 1.343
6	BETTE	NICHOLSON	2006-02-15 09:34:33 5.0
7	GRACE	MOSTEL	2006-02-15 09:34:33 6.0
8	MATTHEW	JOHANSSON	2006-02-15 09:34:33 7.0
9	JOE	SWANK	2006-02-15 09:34:33 8.0
10	CHRISTIAN	GABLE	2006-02-15 09:34:33 9.1
11	ZERO	CAGE	2006-02-15 09:34:33 10.001
12	KARL	BERRY	2017-11-02 19:15:42.308637+08 11.001
A Fateful Reflection of a Waitress And a Boat who must Discover a Sumo Wrestler in Ancient China
Kwara & Kogi
{"Deleted Scenes","Behind the Scenes"}
'awe':5 'awe-inspir':4 'barbarella':1 'cat':13 'conquer':16 'dog':18 'feminist':10 'inspir':6 'monasteri':21 'must':15 'stori':7 'streetcar':2
PHP	₱ USD $
\N  Some other value
\\."#);
    let ast = parse_sql(&sql);
    println!("{:#?}", ast);
    //assert_eq!(sql, ast.to_string());
}

#[test]
fn parse_timestamps_example() {
    let sql = "2016-02-15 09:43:33";
    let _ = parse_sql(sql);
    //TODO add assertion
    //assert_eq!(sql, ast.to_string());
}

#[test]
fn parse_timestamps_with_millis_example() {
    let sql = "2017-11-02 19:15:42.308637";
    let _ = parse_sql(sql);
    //TODO add assertion
    //assert_eq!(sql, ast.to_string());
}

#[test]
fn parse_example_value() {
    let sql = "SARAH.LEWIS@sakilacustomer.org";
    let ast = parse_sql(sql);
    assert_eq!(sql, ast.to_string());
}

#[test]
fn parse_scalar_function_in_projection() {
    let sql = String::from("SELECT sqrt(id) FROM foo");
    let ast = parse_sql(&sql);
    assert_eq!(sql, ast.to_string());
    if let ASTNode::SQLSelect { projection, .. } = ast {
        assert_eq!(
            vec![ASTNode::SQLFunction {
                id: String::from("sqrt"),
                args: vec![ASTNode::SQLIdentifier(String::from("id"))],
            }],
            projection
        );
    } else {
        assert!(false);
    }
}

#[test]
fn parse_aggregate_with_group_by() {
    let sql = String::from("SELECT a, COUNT(1), MIN(b), MAX(b) FROM foo GROUP BY a");
    let ast = parse_sql(&sql);
    assert_eq!(sql, ast.to_string());
}

#[test]
fn parse_literal_string() {
    let sql = "SELECT 'one'";
    let ast = parse_sql(&sql);
    assert_eq!(sql, ast.to_string());
    match ast {
        ASTNode::SQLSelect { ref projection, .. } => {
            assert_eq!(
                projection[0],
                ASTNode::SQLValue(Value::SingleQuotedString("one".to_string()))
            );
        }
        _ => panic!(),
    }
}

#[test]
fn parse_select_version() {
    let sql = "SELECT @@version";
    let ast = parse_sql(&sql);
    assert_eq!(sql, ast.to_string());
    match ast {
        ASTNode::SQLSelect { ref projection, .. } => {
            assert_eq!(
                projection[0],
                ASTNode::SQLIdentifier("@@version".to_string())
            );
        }
        _ => panic!(),
    }
}

#[test]
fn parse_function_now() {
    let sql = "now()";
    let ast = parse_sql(sql);
    assert_eq!(sql, ast.to_string());
}

#[test]
fn parse_implicit_join() {
    let sql = "SELECT * FROM t1, t2";

    match verified(sql) {
        ASTNode::SQLSelect { joins, .. } => {
            assert_eq!(joins.len(), 1);
            assert_eq!(
                joins[0],
                Join {
                    relation: ASTNode::SQLIdentifier("t2".to_string()),
                    join_operator: JoinOperator::Implicit
                }
            )
        }
        _ => assert!(false),
    }
}

#[test]
fn parse_cross_join() {
    let sql = "SELECT * FROM t1 CROSS JOIN t2";

    match verified(sql) {
        ASTNode::SQLSelect { joins, .. } => {
            assert_eq!(joins.len(), 1);
            assert_eq!(
                joins[0],
                Join {
                    relation: ASTNode::SQLIdentifier("t2".to_string()),
                    join_operator: JoinOperator::Cross
                }
            )
        }
        _ => assert!(false),
    }
}

#[test]
fn parse_joins_on() {
    fn join_with_constraint(
        relation: impl Into<String>,
        f: impl Fn(JoinConstraint) -> JoinOperator,
    ) -> Join {
        Join {
            relation: ASTNode::SQLIdentifier(relation.into()),
            join_operator: f(JoinConstraint::On(ASTNode::SQLBinaryExpr {
                left: Box::new(ASTNode::SQLIdentifier("c1".into())),
                op: SQLOperator::Eq,
                right: Box::new(ASTNode::SQLIdentifier("c2".into())),
            })),
        }
    }
    assert_eq!(
        joins_from(verified("SELECT * FROM t1 JOIN t2 ON c1 = c2")),
        vec![join_with_constraint("t2", JoinOperator::Inner)]
    );
    assert_eq!(
        joins_from(verified("SELECT * FROM t1 LEFT JOIN t2 ON c1 = c2")),
        vec![join_with_constraint("t2", JoinOperator::LeftOuter)]
    );
    assert_eq!(
        joins_from(verified("SELECT * FROM t1 RIGHT JOIN t2 ON c1 = c2")),
        vec![join_with_constraint("t2", JoinOperator::RightOuter)]
    );
    assert_eq!(
        joins_from(verified("SELECT * FROM t1 FULL JOIN t2 ON c1 = c2")),
        vec![join_with_constraint("t2", JoinOperator::FullOuter)]
    );
}

#[test]
fn parse_joins_using() {
    fn join_with_constraint(
        relation: impl Into<String>,
        f: impl Fn(JoinConstraint) -> JoinOperator,
    ) -> Join {
        Join {
            relation: ASTNode::SQLIdentifier(relation.into()),
            join_operator: f(JoinConstraint::Using(vec!["c1".into()])),
        }
    }

    assert_eq!(
        joins_from(verified("SELECT * FROM t1 JOIN t2 USING(c1)")),
        vec![join_with_constraint("t2", JoinOperator::Inner)]
    );
    assert_eq!(
        joins_from(verified("SELECT * FROM t1 LEFT JOIN t2 USING(c1)")),
        vec![join_with_constraint("t2", JoinOperator::LeftOuter)]
    );
    assert_eq!(
        joins_from(verified("SELECT * FROM t1 RIGHT JOIN t2 USING(c1)")),
        vec![join_with_constraint("t2", JoinOperator::RightOuter)]
    );
    assert_eq!(
        joins_from(verified("SELECT * FROM t1 FULL JOIN t2 USING(c1)")),
        vec![join_with_constraint("t2", JoinOperator::FullOuter)]
    );
}

#[test]
fn parse_join_syntax_variants() {
    fn parses_to(from: &str, to: &str) {
        assert_eq!(to, &parse_sql(from).to_string())
    }

    parses_to(
        "SELECT c1 FROM t1 INNER JOIN t2 USING(c1)",
        "SELECT c1 FROM t1 JOIN t2 USING(c1)",
    );
    parses_to(
        "SELECT c1 FROM t1 LEFT OUTER JOIN t2 USING(c1)",
        "SELECT c1 FROM t1 LEFT JOIN t2 USING(c1)",
    );
    parses_to(
        "SELECT c1 FROM t1 RIGHT OUTER JOIN t2 USING(c1)",
        "SELECT c1 FROM t1 RIGHT JOIN t2 USING(c1)",
    );
    parses_to(
        "SELECT c1 FROM t1 FULL OUTER JOIN t2 USING(c1)",
        "SELECT c1 FROM t1 FULL JOIN t2 USING(c1)",
    );
}

#[test]
fn parse_complex_join() {
    let sql = "SELECT c1, c2 FROM t1, t4 JOIN t2 ON t2.c = t1.c LEFT JOIN t3 USING(q, c) WHERE t4.c = t1.c";
    assert_eq!(sql, parse_sql(sql).to_string());
}

fn verified(query: &str) -> ASTNode {
    let ast = parse_sql(query);
    assert_eq!(query, &ast.to_string());
    ast
}

fn joins_from(ast: ASTNode) -> Vec<Join> {
    match ast {
        ASTNode::SQLSelect { joins, .. } => joins,
        _ => panic!("Expected SELECT"),
    }
}

fn parse_sql(sql: &str) -> ASTNode {
    debug!("sql: {}", sql);
    let mut parser = parser(sql);
    let ast = parser.parse().unwrap();
    ast
}

fn parser(sql: &str) -> Parser {
    let dialect = PostgreSqlDialect {};
    let mut tokenizer = Tokenizer::new(&dialect, &sql);
    let tokens = tokenizer.tokenize().unwrap();
    debug!("tokens: {:#?}", tokens);
    Parser::new(tokens)
}

#[test]
fn parse_like() {
    let sql = String::from("SELECT * FROM customers WHERE name LIKE '%a'");
    let ast = parse_sql(&sql);
    assert_eq!(sql, ast.to_string());
    match ast {
        ASTNode::SQLSelect { selection, .. } => {
            assert_eq!(
                ASTNode::SQLBinaryExpr {
                    left: Box::new(ASTNode::SQLIdentifier("name".to_string())),
                    op: SQLOperator::Like,
                    right: Box::new(ASTNode::SQLValue(Value::SingleQuotedString(
                        "%a".to_string()
                    ))),
                },
                *selection.unwrap()
            );
        }
        _ => assert!(false),
    }
}

#[test]
fn parse_not_like() {
    let sql = String::from("SELECT * FROM customers WHERE name NOT LIKE '%a'");
    let ast = parse_sql(&sql);
    assert_eq!(sql, ast.to_string());
    match ast {
        ASTNode::SQLSelect { selection, .. } => {
            assert_eq!(
                ASTNode::SQLBinaryExpr {
                    left: Box::new(ASTNode::SQLIdentifier("name".to_string())),
                    op: SQLOperator::NotLike,
                    right: Box::new(ASTNode::SQLValue(Value::SingleQuotedString(
                        "%a".to_string()
                    ))),
                },
                *selection.unwrap()
            );
        }
        _ => assert!(false),
    }
}
