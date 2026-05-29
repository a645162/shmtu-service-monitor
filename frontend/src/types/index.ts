export interface MonitorServer {
  id: number;
  name: string;
  description: string;
  created_at: string;
}

export interface ServiceInstance {
  id: number;
  server_id: number;
  name: string;
  service_type: string;
  base_url: string;
  poll_interval_secs: number;
  created_at: string;
}

export interface ServiceStatus {
  id: number;
  instance_id: number;
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

export interface CreateServerRequest {
  name: string;
  description?: string;
}

export interface CreateInstanceRequest {
  name: string;
  service_type: string;
  base_url: string;
  poll_interval_secs?: number;
}

export interface DashboardSummary {
  total_servers: number;
  total_instances: number;
  healthy_instances: number;
  busy_instances: number;
  unavailable_instances: number;
  servers: ServerDashboardEntry[];
}

export interface ServerDashboardEntry {
  server: MonitorServer;
  instances: InstanceDashboardEntry[];
}

export interface InstanceDashboardEntry {
  instance: ServiceInstance;
  latest_status: ServiceStatus | null;
}

export interface ServerDetail {
  server: MonitorServer;
  instances: InstanceDashboardEntry[];
}
