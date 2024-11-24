$
P = (1 - u - v) A + u B + v C\
P = A + u arrow(A B) + v arrow(A C)\
bold(0) = u arrow(A B) + v arrow(A C) + arrow(P A)\
cases(
    u arrow(A B)_x + v arrow(A C)_x + arrow(P A)_x &= 0\
    u arrow(A B)_y + v arrow(A C)_y + arrow(P A)_y &= 0
)\
cases(
    mat(u, v, 1) mat(arrow(A B)_x; arrow(A C)_x; arrow(P A)_x) = 0\
    mat(u, v, 1) mat(arrow(A B)_y; arrow(A C)_y; arrow(P A)_y) = 0
)
$
It means that we are looking for a vector (u,v,1) that is orthogonal to (ABx,ACx,PAx) and (ABy,ACy,PAy) at the same time!
