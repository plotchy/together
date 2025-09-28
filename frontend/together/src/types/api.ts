// Types based on backend API responses

export interface TogetherAttestation {
  id: string;
  address_1: string;
  address_2: string;
  attestation_timestamp: number;
  tx_hash?: string;
  block_number?: number;
  created_at: string;
}

export interface ConnectionInfo {
  partner_address: string;
  attestation_timestamp: number;
  tx_hash?: string;
  partner_username?: string;
  connection_strength?: number; // Number of times connected with this partner
  has_optimistic?: boolean; // Whether there are unprocessed optimistic connections
}

export interface UserProfile {
  address: string;
  username?: string;
  profile_picture_url?: string;
  total_connections: number;
  recent_connections: ConnectionInfo[];
}

export interface AttestTogetherRequest {
  my_address: string;
  partner_address: string;
  timestamp: number;
  password: string;
  my_username?: string;
  partner_username?: string;
  my_profile_picture_url?: string;
  partner_profile_picture_url?: string;
}

export interface AttestTogetherResponse {
  signature: string;
  nonce: string;
  deadline: number;
}

export interface TogetherError {
  error: string;
}

export interface ApiResponse<T> {
  data?: T;
  error?: string;
}

export interface UserResponse {
  id: number;
  wallet_address: string;
  created_at: string;
}

export interface CreatePendingConnectionRequest {
  to_user_id: number;
}

export interface PendingConnectionResponse {
  id: string;
  from_user_id: number;
  to_user_id: number;
  created_at: string;
  expires_at: string;
}

export interface UserPendingConnectionsResponse {
  outgoing: PendingConnectionResponse[];
  incoming: PendingConnectionResponse[];
}

export interface OptimisticConnectionResponse {
  id: string;
  user_id_1: number;
  user_id_2: number;
  processed: boolean;
  created_at: string;
}

export interface UserOptimisticConnectionsResponse {
  connections: OptimisticConnectionResponse[];
}
