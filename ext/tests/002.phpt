--TEST--
pphp_test1() Basic test
--SKIPIF--
<?php
if (!extension_loaded('pphp')) {
	echo 'skip';
}
?>
--FILE--
<?php 
$ret = pphp_test1();

var_dump($ret);
?>
--EXPECT--
The extension pphp is loaded and working!
NULL
