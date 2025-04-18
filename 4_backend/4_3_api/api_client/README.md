# Rust API client for openapi

No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)


## Overview

This API client was generated by the [OpenAPI Generator](https://openapi-generator.tech) project.  By using the [openapi-spec](https://openapis.org) from a remote server, you can easily generate an API client.

- API version: 0.1.0
- Package version: 0.1.0
- Build package: `org.openapitools.codegen.languages.RustClientCodegen`

## Installation

Put the package under your project folder in a directory named `openapi` and add the following to `Cargo.toml` under `[dependencies]`:

```
openapi = { path = "./openapi" }
```

## Documentation for API Endpoints

All URIs are relative to *http://localhost*

Class | Method | HTTP request | Description
------------ | ------------- | ------------- | -------------
*RoleApi* | [**create_role**](docs/RoleApi.md#create_role) | **POST** /roles | 
*RoleApi* | [**delete_role**](docs/RoleApi.md#delete_role) | **DELETE** /roles/{slug} | 
*RoleApi* | [**get_role**](docs/RoleApi.md#get_role) | **GET** /roles/{slug} | 
*RoleApi* | [**list_roles**](docs/RoleApi.md#list_roles) | **GET** /roles | 
*RoleApi* | [**update_role**](docs/RoleApi.md#update_role) | **PATCH** /roles/{slug} | 
*UserApi* | [**create_user**](docs/UserApi.md#create_user) | **POST** /users | 
*UserApi* | [**delete_user**](docs/UserApi.md#delete_user) | **DELETE** /users/{id} | 
*UserApi* | [**get_user**](docs/UserApi.md#get_user) | **GET** /users/{id} | 
*UserApi* | [**list_users**](docs/UserApi.md#list_users) | **GET** /users | 
*UserApi* | [**update_user**](docs/UserApi.md#update_user) | **PATCH** /users/{id} | 


## Documentation For Models

 - [CreateRole](docs/CreateRole.md)
 - [CreateUser](docs/CreateUser.md)
 - [Permissions](docs/Permissions.md)
 - [RolesPeriodModel](docs/RolesPeriodModel.md)
 - [UpdateRole](docs/UpdateRole.md)
 - [UpdateUser](docs/UpdateUser.md)
 - [UserWithRoles](docs/UserWithRoles.md)
 - [UserWithRolesAllOf](docs/UserWithRolesAllOf.md)
 - [UsersPeriodModel](docs/UsersPeriodModel.md)


To get access to the crate's generated documentation, use:

```
cargo doc --open
```

## Author



