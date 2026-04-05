export interface ShotData {
	id: string;
	shot_number: number;
	speed_mph: number;
	vla_deg: number;
	hla_deg: number;
	backspin_rpm: number;
	sidespin_rpm: number;
	spin_axis_deg: number;
	total_spin_rpm: number;
	club: string;
	confidence: number;
	processing_time_ms: number;
	timestamp: string;
}

export interface DeviceStatus {
	connected: boolean;
	version: string;
	ball_detected: boolean;
	simulator_connected: string | null;
	uptime_seconds: number;
}

export interface SessionStats {
	shot_count: number;
	avg_speed: number;
	avg_vla: number;
	avg_backspin: number;
	avg_carry_estimate: number;
}

export type ClubType =
	| 'Driver'
	| 'Wood3'
	| 'Wood5'
	| 'Hybrid'
	| 'Iron3'
	| 'Iron4'
	| 'Iron5'
	| 'Iron6'
	| 'Iron7'
	| 'Iron8'
	| 'Iron9'
	| 'PitchingWedge'
	| 'GapWedge'
	| 'SandWedge'
	| 'LobWedge'
	| 'Putter';
