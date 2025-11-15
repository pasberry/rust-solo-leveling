// TypeScript API Client for Rust REST API

export interface User {
  id: string;
  name: string;
  email: string;
  created_at: string;
}

export interface CreateUserRequest {
  name: string;
  email: string;
}

export interface UpdateUserRequest {
  name?: string;
  email?: string;
}

export interface ListUsersParams {
  limit?: number;
  offset?: number;
}

export interface ErrorResponse {
  error: string;
  details?: string;
}

export class ApiError extends Error {
  constructor(
    message: string,
    public status: number,
    public details?: ErrorResponse
  ) {
    super(message);
    this.name = 'ApiError';
  }
}

export class ApiClient {
  constructor(private baseUrl: string) {}

  private async request<T>(
    path: string,
    options?: RequestInit
  ): Promise<T> {
    const url = `${this.baseUrl}${path}`;
    const response = await fetch(url, {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        ...options?.headers,
      },
    });

    if (!response.ok) {
      let errorDetails: ErrorResponse | undefined;
      try {
        errorDetails = await response.json();
      } catch {
        // Response body is not JSON
      }

      throw new ApiError(
        `HTTP ${response.status}: ${response.statusText}`,
        response.status,
        errorDetails
      );
    }

    // Handle 204 No Content
    if (response.status === 204) {
      return undefined as T;
    }

    return response.json();
  }

  users = {
    /**
     * Create a new user
     */
    create: async (data: CreateUserRequest): Promise<User> => {
      return this.request<User>('/api/users', {
        method: 'POST',
        body: JSON.stringify(data),
      });
    },

    /**
     * List users with optional pagination
     */
    list: async (params?: ListUsersParams): Promise<User[]> => {
      const query = new URLSearchParams();
      if (params?.limit) query.set('limit', params.limit.toString());
      if (params?.offset) query.set('offset', params.offset.toString());

      const queryString = query.toString();
      const path = queryString ? `/api/users?${queryString}` : '/api/users';

      return this.request<User[]>(path);
    },

    /**
     * Get a single user by ID
     */
    get: async (id: string): Promise<User> => {
      return this.request<User>(`/api/users/${id}`);
    },

    /**
     * Update a user
     */
    update: async (id: string, data: UpdateUserRequest): Promise<User> => {
      return this.request<User>(`/api/users/${id}`, {
        method: 'PUT',
        body: JSON.stringify(data),
      });
    },

    /**
     * Delete a user
     */
    delete: async (id: string): Promise<void> => {
      return this.request<void>(`/api/users/${id}`, {
        method: 'DELETE',
      });
    },
  };

  /**
   * Check server health
   */
  health = async (): Promise<string> => {
    const response = await fetch(`${this.baseUrl}/health`);
    return response.text();
  };
}
