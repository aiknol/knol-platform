import { describe, it, expect } from 'vitest';
import { OPERATIONS, OperationDef } from './operations';

describe('operations', () => {
  it('exports a non-empty array of operations', () => {
    expect(Array.isArray(OPERATIONS)).toBe(true);
    expect(OPERATIONS.length).toBeGreaterThan(0);
  });

  it('every operation has a unique id', () => {
    const ids = OPERATIONS.map((op) => op.id);
    expect(new Set(ids).size).toBe(ids.length);
  });

  it('every operation has required fields', () => {
    for (const op of OPERATIONS) {
      expect(op.id).toBeTruthy();
      expect(op.label).toBeTruthy();
      expect(op.group).toBeTruthy();
      expect(['GET', 'POST', 'PUT', 'DELETE']).toContain(op.method);
      expect(op.pathTemplate).toMatch(/^\//);
      expect(op.description).toBeTruthy();
      expect(Array.isArray(op.pathParams)).toBe(true);
      expect(Array.isArray(op.bodyFields)).toBe(true);
    }
  });

  it('contains Memory, Graph, and Admin groups', () => {
    const groups = new Set(OPERATIONS.map((op) => op.group));
    expect(groups.has('Memory')).toBe(true);
    expect(groups.has('Graph')).toBe(true);
    expect(groups.has('Admin')).toBe(true);
  });

  it('GET operations have no body fields', () => {
    const getOps = OPERATIONS.filter((op) => op.method === 'GET');
    for (const op of getOps) {
      expect(op.bodyFields).toHaveLength(0);
    }
  });

  it('path params reference placeholders in pathTemplate', () => {
    for (const op of OPERATIONS) {
      for (const param of op.pathParams) {
        expect(op.pathTemplate).toContain(`:${param.name}`);
      }
    }
  });

  it('required fields are marked correctly', () => {
    const searchOp = OPERATIONS.find((op) => op.id === 'search-memory')!;
    const queryField = searchOp.bodyFields.find((f) => f.name === 'query')!;
    expect(queryField.required).toBe(true);

    const userIdField = searchOp.bodyFields.find((f) => f.name === 'user_id')!;
    expect(userIdField.required).toBeFalsy();
  });

  it('search-memory is the first operation', () => {
    expect(OPERATIONS[0].id).toBe('search-memory');
  });

  it('field types are valid', () => {
    const validTypes = ['string', 'number', 'textarea', 'json'];
    for (const op of OPERATIONS) {
      for (const field of [...op.pathParams, ...op.bodyFields]) {
        expect(validTypes).toContain(field.type);
      }
    }
  });
});
