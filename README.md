# nom-sql

[![Build Status](https://travis-ci.org/bostontrader/nom-sql.svg)](https://travis-ci.org/bostontrader/nom-sql)

The purpose of this project is to provide a means to safely submit an entire SQL query as a parameter of an HTTP server route handler.  Our specific use-case is for Rust, Rocket, and MySQL, but this should be easily adaptable to your circumstances.

We consider this to be an intermediate step between RESTful APIs and GraphQL.  As you may have noticed, RESTful APIs tend to suffer from the proliferation of end points and query parameters.  Although GraphQL looks to be able to tame this madness, in real-life it's very difficult to efficiently interface GraphQL with relational databases.

So why not just send an SQL query to a RESTful route handler?  Well, probably because you would open the door to all sorts of SQL Injection mischief. But this is, IMHO, such a tantalizing idea that it's worthy of further consideration.


This project takes the following approach:

1. We started by forking [nom-sql](https://github.com/ms705/nom-sql), which is a full-blown SQL parser. An excellent choice if you need God's own full strength SQL.

2. Next, severely hobble the parser to prevent the successful parsing of just ole thing that Zhou Madman might throw at it.  Only a much safer subset of SQL will successfully parse.  For example:

2.1 No CREATE TABLE, CREATE VIEW, DROP TABLE, or SET statements are allowed.

2.2 No comments.

2.3 Only white-listed table and column names are acceptable.  Wherever the parser wants a table or comment name, said name had better be on the list.

2.4 Further safety constraints as we think of them.


Using this project, an http server's route handler can:

1. Accept a String of whatever SQL the user sends as well as an apikey in a separate parameter.

2. Parse the SQL into an AST.  But this is the hobbled version of SQL so hopefully it's safer.

3. Modify the AST to do its own SQL injection of "AND apikey = ?" (with ? being a placeholder for our route handler's apikey input parameter.) in strategic locations of the query.

4. Generate a trustworthy string of SQL that can be subsequently put to good use.

We're still not real comfortable matching wits with the Dr. Evil's of the world, but this is a humble step in that direction.
