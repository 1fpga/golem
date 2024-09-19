# Migrations Directory

This directory contains the migrations for the Golem Frontend database(s).

The migration system is implemented in Rust for performance reasons. Do not look for usage of this directory in JavaScript.

These files will be minimized as part of the build process before being included. Comments will be removed and multiple spaces or tabs will be replaced by a single space.

WARNING: ONLY SQL FILES (files with `.sql` extension) WILL BE INCLUDED IN THE BUILD. DO NOT USE OTHER FILE EXTENSIONS.
