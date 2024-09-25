all: print light_app.tar oci-layout

print: print.c
	bear -- aarch64-linux-gnu-gcc -nostdlib -O2 -static print.c -o print
	aarch64-linux-gnu-strip print

light_app.tar: print Dockerfile
	docker build -t light_app . --platform aarch64
	docker image save light_app -o light_app.tar

oci-layout: light_app.tar
	rm -f blobs/sha256/*
	tar xf light_app.tar
	touch oci-layout

clean:
	rm -f print oci-layout repositories manifest.json light_app.tar
	rm -rf blobs
