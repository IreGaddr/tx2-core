#!/bin/bash
set -e

echo "Building tx2-core for WASM..."

TARGET_DIR="./pkg"
TARGET_NAME="tx2_core"

cargo build --target wasm32-unknown-unknown --no-default-features --features wasm --release

echo "Running wasm-bindgen..."
wasm-bindgen \
    --target web \
    --out-dir "$TARGET_DIR" \
    --out-name "$TARGET_NAME" \
    --typescript \
    ../target/wasm32-unknown-unknown/release/tx2_core.wasm

echo "Generating TypeScript type definitions..."

cat > "$TARGET_DIR/types.ts" << 'EOF'
/**
 * TX2-Core WASM TypeScript Type Definitions
 *
 * These types ensure isomorphic state compatibility between
 * Rust (native/WASM) and TypeScript (web/node) implementations.
 */

export type EntityId = number & { readonly __entityId: unique symbol };

export interface WasmEntity {
  id: number;
}

export interface WasmComponentData {
  id: string;
  data: any;
}

export interface WasmSerializedComponent {
  id: string;
  data: any;
}

export interface WasmSerializedEntity {
  id: number;
  components: WasmSerializedComponent[];
}

export interface WasmWorldSnapshot {
  entities: WasmSerializedEntity[];
  timestamp: number;
}

export interface WasmWorldInterface {
  createEntity(): WasmEntity;
  createEntityWithId(id: number): WasmEntity;
  destroyEntity(entityId: number): boolean;
  hasEntity(entityId: number): boolean;
  getAllEntities(): WasmEntity[];
  addComponent(entityId: number, componentId: string, data: any): void;
  removeComponent(entityId: number, componentId: string): boolean;
  hasComponent(entityId: number, componentId: string): boolean;
  getComponent(entityId: number, componentId: string): any | null;
  getAllComponents(entityId: number): WasmSerializedComponent[];
  createSnapshot(): WasmWorldSnapshot;
  restoreFromSnapshot(snapshot: WasmWorldSnapshot): void;
  clear(): void;
  query(includeComponents: string[], excludeComponents: string[]): number[];
}

export declare class WasmWorld implements WasmWorldInterface {
  constructor();
  createEntity(): WasmEntity;
  createEntityWithId(id: number): WasmEntity;
  destroyEntity(entityId: number): boolean;
  hasEntity(entityId: number): boolean;
  getAllEntities(): WasmEntity[];
  addComponent(entityId: number, componentId: string, data: any): void;
  removeComponent(entityId: number, componentId: string): boolean;
  hasComponent(entityId: number, componentId: string): boolean;
  getComponent(entityId: number, componentId: string): any | null;
  getAllComponents(entityId: number): WasmSerializedComponent[];
  createSnapshot(): WasmWorldSnapshot;
  restoreFromSnapshot(snapshot: WasmWorldSnapshot): void;
  clear(): void;
  query(includeComponents: string[], excludeComponents: string[]): number[];
  free(): void;
}

export declare function get_wasm_version(): string;
export declare function benchmark_entity_creation(count: number): number;

export interface InitOutput {
  WasmWorld: typeof WasmWorld;
  get_wasm_version: typeof get_wasm_version;
  benchmark_entity_creation: typeof benchmark_entity_creation;
}

export default function init(input?: RequestInfo | URL | Response | BufferSource): Promise<InitOutput>;
EOF

echo "Creating package.json for WASM module..."

cat > "$TARGET_DIR/package.json" << EOF
{
  "name": "@tx2/core-wasm",
  "version": "0.1.0",
  "description": "TX2-Core WASM bindings - High-performance ECS engine",
  "type": "module",
  "main": "tx2_core.js",
  "types": "tx2_core.d.ts",
  "files": [
    "tx2_core.js",
    "tx2_core.d.ts",
    "tx2_core_bg.wasm",
    "tx2_core_bg.wasm.d.ts",
    "types.ts"
  ],
  "sideEffects": [
    "tx2_core.js"
  ],
  "keywords": [
    "ecs",
    "wasm",
    "entity-component-system",
    "game-engine",
    "isomorphic",
    "tx2"
  ],
  "author": "TX-2 Contributors",
  "license": "MIT",
  "repository": {
    "type": "git",
    "url": "https://github.com/IreGaddr/tx2-core.git"
  }
}
EOF

echo "WASM build complete! Output in $TARGET_DIR/"
echo "Files generated:"
ls -lh "$TARGET_DIR/"
