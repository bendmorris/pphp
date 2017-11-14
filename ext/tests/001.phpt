--TEST--
Check if pphp is loaded
--SKIPIF--
<?php
if (!extension_loaded('pphp')) {
	echo 'skip';
}
?>
--FILE--
<?php 
echo 'The extension "pphp" is available';
?>
--EXPECT--
The extension "pphp" is available
