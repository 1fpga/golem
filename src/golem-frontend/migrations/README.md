# Migrations Directory

This directory contains the migrations for the Golem Frontend database(s).

The migrations are exported in JavaScript using a rollup plugin. The module name is `@:migrations`.

These files will be minimized as part of the build process before being included. Comments will be removed and multiple spaces or tabs will be replaced by a single space.

WARNING: ONLY SQL FILES (files with `.sql` extension) WILL BE INCLUDED IN THE BUILD. DO NOT USE OTHER FILE EXTENSIONS.
