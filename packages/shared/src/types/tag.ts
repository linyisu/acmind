export interface Tag {
  id: number;
  user_id: number;
  name: string;
}

export interface CreateTagRequest {
  name: string;
}
