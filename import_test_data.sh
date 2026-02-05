#!/bin/bash
# Import script for test data
# Update YOUR_API_KEY_HERE with your API key that has write permissions

curl -X POST 'https://fra.cloud.appwrite.io/v1/databases/threed-dating-db/collections/dating-profiles/documents/test_profile_0000' \
  -H "X-Appwrite-Key: YOUR_API_KEY_HERE" \
  -H "X-Appwrite-Project: 6899062700398ffeae4f" \
  -H "Content-Type: application/json" \
  -d '{\"adminNotes\":null,\"age\":61,\"createdAt\":\"1770231834000\",\"description\":\"Test profile from Düsseldorf, looking for connections!\",\"gender\":\"agender\",\"hairColor\":\"black\",\"heightCm\":153,\"imageFileIds\":[],\"isActive\":true,\"isTimeout\":false,\"isVerified\":false,\"latitude\":51.14689289201212,\"longitude\":6.692692892012118,\"modelFileId\":null,\"name\":\"Sam 0\",\"pb\":[],\"reason\":null,\"sportsPreferences\":[\"skating\",\"badminton\"],\"updatedAt\":\"1770231834000\",\"userId\":\"test_user_0000\"}'

curl -X POST 'https://fra.cloud.appwrite.io/v1/databases/threed-dating-db/collections/dating-profiles/documents/test_profile_0001' \
  -H "X-Appwrite-Key: YOUR_API_KEY_HERE" \
  -H "X-Appwrite-Project: 6899062700398ffeae4f" \
  -H "Content-Type: application/json" \
  -d '{\"adminNotes\":null,\"age\":40,\"createdAt\":\"1770231834000\",\"description\":\"Test profile from Frankfurt, looking for connections!\",\"gender\":\"other\",\"hairColor\":\"red\",\"heightCm\":210,\"imageFileIds\":[],\"isActive\":true,\"isTimeout\":false,\"isVerified\":false,\"latitude\":50.030092892012135,\"longitude\":8.601292892012133,\"modelFileId\":null,\"name\":\"Sam 1\",\"pb\":[],\"reason\":null,\"sportsPreferences\":[\"climbing\",\"tennis\"],\"updatedAt\":\"1770231834000\",\"userId\":\"test_user_0001\"}'

curl -X POST 'https://fra.cloud.appwrite.io/v1/databases/threed-dating-db/collections/dating-profiles/documents/test_profile_0002' \
  -H "X-Appwrite-Key: YOUR_API_KEY_HERE" \
  -H "X-Appwrite-Project: 6899062700398ffeae4f" \
  -H "Content-Type: application/json" \
  -d '{\"adminNotes\":null,\"age\":80,\"createdAt\":\"1770231834000\",\"description\":\"Test profile from Hamburg, looking for connections!\",\"gender\":\"non_binary\",\"hairColor\":\"blonde\",\"heightCm\":151,\"imageFileIds\":[],\"isActive\":true,\"isTimeout\":false,\"isVerified\":false,\"latitude\":53.47029289201214,\"longitude\":9.912892892012145,\"modelFileId\":null,\"name\":\"Xavier 2\",\"pb\":[],\"reason\":null,\"sportsPreferences\":[\"golf\",\"yoga\",\"boxing\",\"skating\"],\"updatedAt\":\"1770231834000\",\"userId\":\"test_user_0002\"}'

curl -X POST 'https://fra.cloud.appwrite.io/v1/databases/threed-dating-db/collections/dating-profiles/documents/test_profile_0003' \
  -H "X-Appwrite-Key: YOUR_API_KEY_HERE" \
  -H "X-Appwrite-Project: 6899062700398ffeae4f" \
  -H "Content-Type: application/json" \
  -d '{\"adminNotes\":null,\"age\":44,\"createdAt\":\"1770231834000\",\"description\":\"Test profile from Cologne, looking for connections!\",\"gender\":\"agender\",\"hairColor\":\"white\",\"heightCm\":182,\"imageFileIds\":[],\"isActive\":true,\"isTimeout\":false,\"isVerified\":false,\"latitude\":50.856692892012155,\"longitude\":6.879492892012156,\"modelFileId\":null,\"name\":\"Luna 3\",\"pb\":[],\"reason\":null,\"sportsPreferences\":[\"volleyball\",\"gym\",\"dancing\"],\"updatedAt\":\"1770231834000\",\"userId\":\"test_user_0003\"}'

curl -X POST 'https://fra.cloud.appwrite.io/v1/databases/threed-dating-db/collections/dating-profiles/documents/test_profile_0004' \
  -H "X-Appwrite-Key: YOUR_API_KEY_HERE" \
  -H "X-Appwrite-Project: 6899062700398ffeae4f" \
  -H "Content-Type: application/json" \
  -d '{\"adminNotes\":null,\"age\":28,\"createdAt\":\"1770231834000\",\"description\":\"Test profile from Stuttgart, looking for connections!\",\"gender\":\"other\",\"hairColor\":\"blonde\",\"heightCm\":208,\"imageFileIds\":[],\"isActive\":true,\"isTimeout\":false,\"isVerified\":true,\"latitude\":48.694992892012166,\"longitude\":9.102092892012166,\"modelFileId\":null,\"name\":\"Grayson 4\",\"pb\":[],\"reason\":null,\"sportsPreferences\":[\"boxing\",\"climbing\",\"martial_arts\",\"dancing\",\"swimming\"],\"updatedAt\":\"1770231834000\",\"userId\":\"test_user_0004\"}'

curl -X POST 'https://fra.cloud.appwrite.io/v1/databases/threed-dating-db/collections/dating-profiles/documents/test_profile_0005' \
  -H "X-Appwrite-Key: YOUR_API_KEY_HERE" \
  -H "X-Appwrite-Project: 6899062700398ffeae4f" \
  -H "Content-Type: application/json" \
  -d '{\"adminNotes\":null,\"age\":69,\"createdAt\":\"1770231834000\",\"description\":\"Test profile from Düsseldorf, looking for connections!\",\"gender\":\"non_binary\",\"hairColor\":\"blonde\",\"heightCm\":178,\"imageFileIds\":[],\"isActive\":true,\"isTimeout\":false,\"isVerified\":false,\"latitude\":51.146892892012175,\"longitude\":6.692692892012178,\"modelFileId\":null,\"name\":\"Jade 5\",\"pb\":[],\"reason\":null,\"sportsPreferences\":[\"martial_arts\",\"cycling\",\"surfing\",\"soccer\"],\"updatedAt\":\"1770231834000\",\"userId\":\"test_user_0005\"}'

curl -X POST 'https://fra.cloud.appwrite.io/v1/databases/threed-dating-db/collections/dating-profiles/documents/test_profile_0006' \
  -H "X-Appwrite-Key: YOUR_API_KEY_HERE" \
  -H "X-Appwrite-Project: 6899062700398ffeae4f" \
  -H "Content-Type: application/json" \
  -d '{\"adminNotes\":null,\"age\":19,\"createdAt\":\"1770231834000\",\"description\":\"Test profile from Stuttgart, looking for connections!\",\"gender\":\"male\",\"hairColor\":\"black\",\"heightCm\":158,\"imageFileIds\":[],\"isActive\":true,\"isTimeout\":false,\"isVerified\":false,\"latitude\":48.69499289201219,\"longitude\":9.102092892012188,\"modelFileId\":null,\"name\":\"Sage 6\",\"pb\":[],\"reason\":null,\"sportsPreferences\":[\"yoga\",\"badminton\",\"skiing\",\"hiking\"],\"updatedAt\":\"1770231834000\",\"userId\":\"test_user_0006\"}'

curl -X POST 'https://fra.cloud.appwrite.io/v1/databases/threed-dating-db/collections/dating-profiles/documents/test_profile_0007' \
  -H "X-Appwrite-Key: YOUR_API_KEY_HERE" \
  -H "X-Appwrite-Project: 6899062700398ffeae4f" \
  -H "Content-Type: application/json" \
  -d '{\"adminNotes\":null,\"age\":19,\"createdAt\":\"1770231834000\",\"description\":\"Test profile from Berlin, looking for connections!\",\"gender\":\"female\",\"hairColor\":\"white\",\"heightCm\":196,\"imageFileIds\":[],\"isActive\":true,\"isTimeout\":false,\"isVerified\":false,\"latitude\":52.4391928920122,\"longitude\":13.3241928920122,\"modelFileId\":null,\"name\":\"Frankie 7\",\"pb\":[],\"reason\":null,\"sportsPreferences\":[\"soccer\",\"cycling\"],\"updatedAt\":\"1770231834000\",\"userId\":\"test_user_0007\"}'

curl -X POST 'https://fra.cloud.appwrite.io/v1/databases/threed-dating-db/collections/dating-profiles/documents/test_profile_0008' \
  -H "X-Appwrite-Key: YOUR_API_KEY_HERE" \
  -H "X-Appwrite-Project: 6899062700398ffeae4f" \
  -H "Content-Type: application/json" \
  -d '{\"adminNotes\":null,\"age\":54,\"createdAt\":\"1770231834000\",\"description\":\"Test profile from Frankfurt, looking for connections!\",\"gender\":\"male\",\"hairColor\":\"blonde\",\"heightCm\":200,\"imageFileIds\":[],\"isActive\":true,\"isTimeout\":false,\"isVerified\":true,\"latitude\":50.03009289201221,\"longitude\":8.60129289201221,\"modelFileId\":null,\"name\":\"Onyx 8\",\"pb\":[],\"reason\":null,\"sportsPreferences\":[\"football\",\"skiing\"],\"updatedAt\":\"1770231834000\",\"userId\":\"test_user_0008\"}'

curl -X POST 'https://fra.cloud.appwrite.io/v1/databases/threed-dating-db/collections/dating-profiles/documents/test_profile_0009' \
  -H "X-Appwrite-Key: YOUR_API_KEY_HERE" \
  -H "X-Appwrite-Project: 6899062700398ffeae4f" \
  -H "Content-Type: application/json" \
  -d '{\"adminNotes\":null,\"age\":66,\"createdAt\":\"1770231834000\",\"description\":\"Test profile from Frankfurt, looking for connections!\",\"gender\":\"agender\",\"hairColor\":\"gray\",\"heightCm\":212,\"imageFileIds\":[],\"isActive\":true,\"isTimeout\":false,\"isVerified\":false,\"latitude\":50.03009289201222,\"longitude\":8.601292892012221,\"modelFileId\":null,\"name\":\"Tatum 9\",\"pb\":[],\"reason\":null,\"sportsPreferences\":[\"surfing\",\"gym\",\"golf\",\"hiking\",\"football\"],\"updatedAt\":\"1770231834000\",\"userId\":\"test_user_0009\"}'

