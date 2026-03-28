#include <stdio.h>
#include <windows.h>
void test_recursion(int a){
    if(a == 1){
        printf("current is %d\n", a);

    } else {
        printf("current is %d\n", a);
        test_recursion(a - 1);
    }
}

int main(void) {
    int a = 12;
    test_recursion(a);
}
