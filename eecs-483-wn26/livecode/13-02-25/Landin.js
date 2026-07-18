factBox = undefined;
fact = (x) => {
    if (x == 0) { return 1 }
    else { return (x * factBox(x - 1))}
};
factBox = fact;
