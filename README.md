# nom-sql

[![Build Status](https://travis-ci.org/bostontrader/nom-sql.svg)](https://travis-ci.org/bostontrader/nom-sql)

The purpose of this project is to provide a means to safely submit an entire SQL SELECT query as a parameter of an HTTP server route handler.  Our specific use-case is for Rust, Rocket, and MySQL, but this should be easily adaptable to your circumstances.

We consider this to be an intermediate step between RESTful APIs and GraphQL.  As you may have noticed, RESTful APIs tend to suffer from the proliferation of end points and query parameters.  Although GraphQL looks to be able to tame this madness, in real-life it's very difficult to efficiently interface GraphQL with relational databases.

So why not just send an SQL query to a RESTful route handler?  Well, probably because you would open the door to all sorts of SQL Injection mischief. But this is, IMHO, such a tantalizing idea that it's worthy of further consideration.


This project takes the following approach:

1. We started by forking [nom-sql](https://github.com/ms705/nom-sql), which is a full-blown SQL parser. An excellent choice if you need God's own full strength SQL.

2. Next, severely hobble the parser to prevent the successful parsing of just ole thing that Zhou Madman might throw at it.  Only SQL SELECT statements will successfully parse and the column and table names in said statement must all be included on a white list.

Using this project, an http server's route handler can:

1. Accept SQL SELECT statement the user sends as well as an apikey in a separate parameter.

2. Parse the SQL into an AST.

3. Modify the AST to do its own SQL injection of "AND apikey = ?" (with ? being a placeholder for our route handler's apikey input parameter.) in strategic locations of the query.

4. Generate a trustworthy string of SQL that can be subsequently put to good use.

We're still not real comfortable matching wits with the Dr. Evil's of the world, but this is a humble step in that direction.
