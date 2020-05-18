script {
use 0x0::System;
// Script for adding a new validator
// Will only succeed when run by the Association address

fun main(new_validator: address) {
  System::remove_validator(new_validator)
}
}
