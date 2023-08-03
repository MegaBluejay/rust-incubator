# \RoleApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_role**](RoleApi.md#create_role) | **POST** /roles | 
[**delete_role**](RoleApi.md#delete_role) | **DELETE** /roles/{slug} | 
[**get_role**](RoleApi.md#get_role) | **GET** /roles/{slug} | 
[**list_roles**](RoleApi.md#list_roles) | **GET** /roles | 
[**update_role**](RoleApi.md#update_role) | **PATCH** /roles/{slug} | 



## create_role

> crate::models::RolesPeriodModel create_role(create_role)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_role** | [**CreateRole**](CreateRole.md) |  | [required] |

### Return type

[**crate::models::RolesPeriodModel**](roles.Model.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_role

> delete_role(slug)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**slug** | **String** | Role slug | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_role

> crate::models::RolesPeriodModel get_role(slug)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**slug** | **String** | Role slug | [required] |

### Return type

[**crate::models::RolesPeriodModel**](roles.Model.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_roles

> Vec<crate::models::RolesPeriodModel> list_roles()


### Parameters

This endpoint does not need any parameter.

### Return type

[**Vec<crate::models::RolesPeriodModel>**](roles.Model.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_role

> crate::models::RolesPeriodModel update_role(slug, update_role)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**slug** | **String** | Role slug | [required] |
**update_role** | [**UpdateRole**](UpdateRole.md) |  | [required] |

### Return type

[**crate::models::RolesPeriodModel**](roles.Model.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

