-- Add migration script here
   CREATE TABLE users (
       id SERIAL PRIMARY KEY,
       username VARCHAR(32) NOT NULL UNIQUE,
       email VARCHAR(255) NOT NULL UNIQUE,
       password_hash VARCHAR(255) NOT NULL,
       created_at TIMESTAMP WITH TIME ZONE DEFAULT now()
   );