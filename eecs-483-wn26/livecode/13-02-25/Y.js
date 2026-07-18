Y = (f) => { let s = (x) => (v) => f(x(x))(v); return s(s)};

fac = Y((fac) => (x) => {
    if (x == 0) { return 1 }
    else { return (x * fac(x - 1))}
})
