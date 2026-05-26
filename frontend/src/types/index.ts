export interface Service {
  id: number;
  name: string;
  service_type: string;
  base_url: string;
  poll_interval_secs: number;
  created_at: string;
}

export interface ServiceStatus {
  id: number;
  service_id: number;
  status: 'healthy' | 'busy' | 'unavailable';
  availability_level: string;
  models_loaded: boolean;
  pending_requests: number;
  queue_capacity: number;
  utilization_percent: number;
  avg_response_ms: number | null;
  total_requests: number | null;
  success_count: number | null;
  failure_count: number | null;
  polled_at: string;
  response_time_ms: number;
}

export interface CreateServiceRequest {
  name: string;
  service_type: string;
  base_url: string;
  poll_interval_secs?: number;
}

export interface DashboardSummary {
  total_services: number;
  healthy_services: number;
  busy_services: number;
  unavailable_services: number;
  services: ServiceDashboardEntry[];
}

export interface ServiceDashboardEntry {
  service: Service;
  latest_status: ServiceStatus | null;
}
