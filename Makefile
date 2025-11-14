
CC = cargo

all: um


um: src/*
	echo "Na na na na na! Naomi and Vedant rule 😈!";
	$(CC) build --release; mv target/release/um .;
