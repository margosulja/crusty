int give_value_please() {
    return 42;
}

int main() {
    int value = give_value_please();
    printf("the value i got from the generous function: %d\n", value);
}
