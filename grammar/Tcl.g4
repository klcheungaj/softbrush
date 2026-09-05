grammar Tcl;

// Tcl has a deliberately small language grammar. Commands interpret their
// words after this recursive substitution and grouping pass.
script
    : scriptItem* EOF
    ;

scriptItem
    : bracedWord
    | quotedWord
    | commandSubstitution
    | atom
    ;

bracedWord
    : LBRACE bracedItem* RBRACE
    ;

bracedItem
    : bracedWord
    | LBRACKET
    | RBRACKET
    | DQUOTE
    | atom
    ;

quotedWord
    : DQUOTE quotedItem* DQUOTE
    ;

quotedItem
    : commandSubstitution
    | LBRACE
    | RBRACE
    | RBRACKET
    | atom
    ;

commandSubstitution
    : LBRACKET scriptItem* RBRACKET
    ;

atom
    : VARIABLE
    | NUMBER
    | OPTION
    | TEXT
    | WS
    | NEWLINE
    | SEMI
    | HASH
    | LINE_CONTINUATION
    | ESCAPE
    | DOLLAR
    | BACKSLASH
    ;

LBRACE : '{';
RBRACE : '}';
LBRACKET : '[';
RBRACKET : ']';
DQUOTE : '"';
SEMI : ';';
HASH : '#';

LINE_CONTINUATION
    : '\\' '\r'? '\n' [ \t]*
    ;

VARIABLE
    : '$' ('{' ~[}\r\n]* '}' | [A-Za-z0-9_:]+ ('(' ~[)\r\n]* ')')?)
    ;

NUMBER
    : [+-]? ([0-9]+ ('.' [0-9]*)? | '.' [0-9]+) ([eE] [+-]? [0-9]+)?
    ;

OPTION
    : '-' [A-Za-z_] [A-Za-z0-9_-]*
    ;

TEXT
    : ~[{}\u005B\u005D"$;#\\ \t\r\n]+
    ;

WS : [ \t\f]+;
NEWLINE : '\r'? '\n' | '\r';
ESCAPE : '\\' .;
DOLLAR : '$';
BACKSLASH : '\\';
