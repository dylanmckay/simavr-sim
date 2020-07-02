#include "../../../libavrlit/avr-lit.hpp"

using namespace test;

/// Computes the factorial of a number.
unsigned long long factorial(unsigned long long n) {
  auto fac = 1;

  for(auto i=1; i<=n; i++)
    fac *= i;

  return fac;
}

void run_test() {
  for (unsigned long long i=0; i<=7; i++) {
    call(factorial, i);
  }
}

