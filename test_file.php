<?php

echo "hello world" . "\n";

class A {}

class B {}

$a = new A;

echo (is_a($a, "A") ? "is an A" : "not an A") . "\n";
echo (is_a($a, "B") ? "is a B" : "not a B") . "\n";

$n = 1;
$n = $n + 1;
$m = $n - 1;
echo "$n\n";
