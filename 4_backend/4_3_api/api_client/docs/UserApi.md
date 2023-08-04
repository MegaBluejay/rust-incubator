# \UserApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_user**](UserApi.md#create_user) | **POST** /users | 
[**delete_user**](UserApi.md#delete_user) | **DELETE** /users/{id} | 
[**get_user**](UserApi.md#get_user) | **GET** /users/{id} | 
[**list_users**](UserApi.md#list_users) | **GET** /users | 
[**update_user**](UserApi.md#update_user) | **PATCH** /users/{id} | 



## create_user

> crate::models::UserWithRoles create_user(create_user)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_user** | [**CreateUser**](CreateUser.md) |  | [required] |

### Return type

[**crate::models::UserWithRoles**](UserWithRoles.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_user

> delete_user(id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i32** | User id | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_user

> crate::models::UserWithRoles get_user(id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i32** | User id | [required] |

### Return type

[**crate::models::UserWithRoles**](UserWithRoles.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_users

> Vec<crate::models::UserWithRoles> list_users()


### Parameters

This endpoint does not need any parameter.

### Return type

[**Vec<crate::models::UserWithRoles>**](UserWithRoles.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_user

> crate::models::UserWithRoles update_user(id, update_user)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i32** | User id | [required] |
**update_user** | [**UpdateUser**](UpdateUser.md) |  | [required] |

### Return type

[**crate::models::UserWithRoles**](UserWithRoles.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

