export type Node<T> = {
  id: number,
  data: T
};

export type Edge = {
  from: number,
  to: number,
};

export type Graph<T> = {
  nodes: Node<T>[],
  edges: Edge[],
};
