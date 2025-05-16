# API Documentation

## Overview
This document details the REST API endpoints and WebSocket interfaces for the Task Manager.

## Authentication
All API endpoints (except login/register) require a valid JWT token in the Authorization header:
```
Authorization: Bearer <token>
```

## REST Endpoints

### Authentication
- `POST /api/auth/register` - Register new user
- `POST /api/auth/login` - User login
- `POST /api/auth/refresh` - Refresh token
- `POST /api/auth/logout` - User logout

### Users
- `GET /api/users/me` - Get current user
- `PUT /api/users/me` - Update current user
- `GET /api/users/{id}` - Get user by ID
- `GET /api/users` - List users (admin only)

### Tasks
- `GET /api/tasks` - List tasks
- `POST /api/tasks` - Create task
- `GET /api/tasks/{id}` - Get task
- `PUT /api/tasks/{id}` - Update task
- `DELETE /api/tasks/{id}` - Delete task
- `POST /api/tasks/{id}/assign` - Assign task
- `POST /api/tasks/{id}/complete` - Complete task

### Teams
- `GET /api/teams` - List teams
- `POST /api/teams` - Create team
- `GET /api/teams/{id}` - Get team
- `PUT /api/teams/{id}` - Update team
- `DELETE /api/teams/{id}` - Delete team
- `POST /api/teams/{id}/members` - Add team member
- `DELETE /api/teams/{id}/members/{userId}` - Remove team member

### Files
- `POST /api/files` - Upload file
- `GET /api/files/{id}` - Download file
- `DELETE /api/files/{id}` - Delete file

## WebSocket Interface

### Connection
```
ws://<host>/ws?token=<jwt_token>
```

### Events
- `task.created` - New task created
- `task.updated` - Task updated
- `task.deleted` - Task deleted
- `task.assigned` - Task assigned
- `task.completed` - Task completed
- `team.updated` - Team updated
- `notification` - New notification

## Error Responses
All error responses follow this format:
```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human readable message"
  }
}
```

## Rate Limiting
- 100 requests per minute per IP
- 1000 requests per hour per user

## Versioning
API versioning is handled through the URL path:
```
/api/v1/...
```

## Data Models
Detailed request/response models will be documented in a separate file. 