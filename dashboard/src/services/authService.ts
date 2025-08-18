import axios from 'axios';

const API_BASE_URL = process.env.REACT_APP_API_URL || 'http://localhost:3000/api';

interface LoginResponse {
  token: string;
  user: {
    user_id: string;
    username: string;
    email: string;
    created_at: string;
    updated_at: string;
    is_active: boolean;
  };
}

interface User {
  user_id: string;
  username: string;
  email: string;
  created_at: string;
  updated_at: string;
  is_active: boolean;
}

class AuthService {
  private axiosInstance = axios.create({
    baseURL: API_BASE_URL,
    timeout: 10000,
  });

  constructor() {
    // Add request interceptor to include token
    this.axiosInstance.interceptors.request.use((config) => {
      const token = localStorage.getItem('token');
      if (token) {
        config.headers.Authorization = `Bearer ${token}`;
      }
      return config;
    });

    // Add response interceptor for error handling
    this.axiosInstance.interceptors.response.use(
      (response) => response,
      (error) => {
        if (error.response?.status === 401) {
          localStorage.removeItem('token');
          window.location.href = '/login';
        }
        return Promise.reject(error);
      }
    );
  }

  async login(username: string, password: string): Promise<LoginResponse> {
    const response = await this.axiosInstance.post('/auth/login', {
      username,
      password,
    });
    return response.data;
  }

  async register(username: string, email: string, password: string): Promise<User> {
    const response = await this.axiosInstance.post('/auth/register', {
      username,
      email,
      password,
    });
    return response.data;
  }

  async validateToken(token: string): Promise<User | null> {
    try {
      const response = await this.axiosInstance.get('/auth/me', {
        headers: {
          Authorization: `Bearer ${token}`,
        },
      });
      return response.data;
    } catch {
      return null;
    }
  }

  async refreshToken(): Promise<string | null> {
    try {
      const response = await this.axiosInstance.post('/auth/refresh');
      return response.data.token;
    } catch {
      return null;
    }
  }
}

export const authService = new AuthService();