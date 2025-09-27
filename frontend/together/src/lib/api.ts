import { 
  UserProfile, 
  TogetherAttestation, 
  AttestTogetherRequest, 
  AttestTogetherResponse,
  TogetherError,
  ApiResponse
} from '@/types/api';

const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'https://api.togetherapp.app';

class ApiClient {
  private baseUrl: string;

  constructor(baseUrl: string = API_BASE_URL) {
    this.baseUrl = baseUrl;
  }

  private async request<T>(
    endpoint: string, 
    options: RequestInit = {}
  ): Promise<ApiResponse<T>> {
    try {
      const response = await fetch(`${this.baseUrl}${endpoint}`, {
        headers: {
          'Content-Type': 'application/json',
          ...options.headers,
        },
        ...options,
      });

      if (!response.ok) {
        const errorData: TogetherError = await response.json().catch(() => ({ 
          error: `HTTP ${response.status}: ${response.statusText}` 
        }));
        return { error: errorData.error };
      }

      const data = await response.json();
      return { data };
    } catch (error) {
      return { 
        error: error instanceof Error ? error.message : 'Network error' 
      };
    }
  }

  async getUserProfile(
    address: string, 
    params?: { 
      limit?: number; 
      username?: string; 
      profile_picture_url?: string; 
    }
  ): Promise<ApiResponse<UserProfile>> {
    const searchParams = new URLSearchParams();
    if (params?.limit) searchParams.append('limit', params.limit.toString());
    if (params?.username) searchParams.append('username', params.username);
    if (params?.profile_picture_url) searchParams.append('profile_picture_url', params.profile_picture_url);
    
    const query = searchParams.toString();
    const endpoint = `/api/profile/${address}${query ? `?${query}` : ''}`;
    
    return this.request<UserProfile>(endpoint);
  }

  async checkTogether(
    address1: string, 
    address2: string
  ): Promise<ApiResponse<TogetherAttestation | null>> {
    const endpoint = `/api/check-together/${address1}?address_2=${address2}`;
    return this.request<TogetherAttestation | null>(endpoint);
  }

  async attestTogether(
    request: AttestTogetherRequest
  ): Promise<ApiResponse<AttestTogetherResponse>> {
    return this.request<AttestTogetherResponse>('/api/attest', {
      method: 'POST',
      body: JSON.stringify(request),
    });
  }

  async health(): Promise<ApiResponse<string>> {
    return this.request<string>('/health');
  }
}

export const apiClient = new ApiClient();
export { ApiClient };
