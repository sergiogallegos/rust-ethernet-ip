import React, { useState } from 'react';
import './TagOperations.css';
import { PlcValueJson, DATA_TYPES, DataType } from '../types';

interface TagOperationsProps {
  onReadTag: (tagName: string) => Promise<{
    value: PlcValueJson | null;
    dataType: string | null;
    error: string | null;
  }>;
  onWriteTag: (tagName: string, value: PlcValueJson) => Promise<{
    success: boolean;
    error: string | null;
  }>;
}

const TagOperations: React.FC<TagOperationsProps> = ({
  onReadTag,
  onWriteTag,
}) => {
  const [readTagName, setReadTagName] = useState('');
  const [readResult, setReadResult] = useState<{
    value: PlcValueJson | null;
    dataType: string | null;
    error: string | null;
  } | null>(null);
  const [readLoading, setReadLoading] = useState(false);

  const [writeTagName, setWriteTagName] = useState('');
  const [writeDataType, setWriteDataType] = useState<DataType>('DINT');
  const [writeValue, setWriteValue] = useState('');
  const [writeResult, setWriteResult] = useState<{
    success: boolean;
    error: string | null;
  } | null>(null);
  const [writeLoading, setWriteLoading] = useState(false);

  const handleRead = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!readTagName.trim()) return;

    setReadLoading(true);
    setReadResult(null);
    const result = await onReadTag(readTagName.trim());
    setReadResult(result);
    setReadLoading(false);
  };

  const handleWrite = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!writeTagName.trim() || !writeValue.trim()) return;

    setWriteLoading(true);
    setWriteResult(null);

    // Convert value based on data type
    let plcValue: PlcValueJson;
    try {
      switch (writeDataType) {
        case 'BOOL':
          plcValue = {
            type: 'BOOL',
            value: writeValue.toLowerCase() === 'true' || writeValue === '1',
          };
          break;
        case 'SINT':
          plcValue = { type: 'SINT', value: parseInt(writeValue) };
          break;
        case 'INT':
          plcValue = { type: 'INT', value: parseInt(writeValue) };
          break;
        case 'DINT':
          plcValue = { type: 'DINT', value: parseInt(writeValue) };
          break;
        case 'LINT':
          plcValue = { type: 'LINT', value: parseInt(writeValue) };
          break;
        case 'USINT':
          plcValue = { type: 'USINT', value: parseInt(writeValue) };
          break;
        case 'UINT':
          plcValue = { type: 'UINT', value: parseInt(writeValue) };
          break;
        case 'UDINT':
          plcValue = { type: 'UDINT', value: parseInt(writeValue) };
          break;
        case 'ULINT':
          plcValue = { type: 'ULINT', value: parseInt(writeValue) };
          break;
        case 'REAL':
          plcValue = { type: 'REAL', value: parseFloat(writeValue) };
          break;
        case 'LREAL':
          plcValue = { type: 'LREAL', value: parseFloat(writeValue) };
          break;
        case 'STRING':
          plcValue = { type: 'STRING', value: writeValue };
          break;
        default:
          throw new Error('Invalid data type');
      }
    } catch (err) {
      setWriteResult({
        success: false,
        error: 'Invalid value format',
      });
      setWriteLoading(false);
      return;
    }

    const result = await onWriteTag(writeTagName.trim(), plcValue);
    setWriteResult(result);
    setWriteLoading(false);
  };

  const formatValue = (value: PlcValueJson): string => {
    switch (value.type) {
      case 'BOOL':
        return value.value ? 'true' : 'false';
      case 'STRING':
        return value.value;
      default:
        return String(value.value);
    }
  };

  return (
    <div className="tag-operations">
      <div className="operations-grid">
        {/* Read Section */}
        <div className="operation-section">
          <h2>📖 Read Tag</h2>
          <form onSubmit={handleRead} className="tag-form">
            <div className="form-group">
              <label htmlFor="read-tag">Tag Name:</label>
              <input
                id="read-tag"
                type="text"
                value={readTagName}
                onChange={(e) => setReadTagName(e.target.value)}
                placeholder="TestDINT"
                disabled={readLoading}
                required
              />
            </div>
            <button
              type="submit"
              disabled={readLoading}
              className="btn btn-primary"
            >
              {readLoading ? 'Reading...' : 'Read Tag'}
            </button>
          </form>

          {readResult && (
            <div
              className={`result ${
                readResult.error ? 'result-error' : 'result-success'
              }`}
            >
              {readResult.error ? (
                <div>
                  <strong>Error:</strong> {readResult.error}
                </div>
              ) : (
                <div>
                  <div>
                    <strong>Value:</strong>{' '}
                    {readResult.value
                      ? formatValue(readResult.value)
                      : 'N/A'}
                  </div>
                  {readResult.dataType && (
                    <div>
                      <strong>Type:</strong> {readResult.dataType}
                    </div>
                  )}
                </div>
              )}
            </div>
          )}
        </div>

        {/* Write Section */}
        <div className="operation-section">
          <h2>✏️ Write Tag</h2>
          <form onSubmit={handleWrite} className="tag-form">
            <div className="form-group">
              <label htmlFor="write-tag">Tag Name:</label>
              <input
                id="write-tag"
                type="text"
                value={writeTagName}
                onChange={(e) => setWriteTagName(e.target.value)}
                placeholder="TestDINT"
                disabled={writeLoading}
                required
              />
            </div>
            <div className="form-group">
              <label htmlFor="data-type">Data Type:</label>
              <select
                id="data-type"
                value={writeDataType}
                onChange={(e) => setWriteDataType(e.target.value as DataType)}
                disabled={writeLoading}
                className="select-input"
              >
                {DATA_TYPES.map((type) => (
                  <option key={type} value={type}>
                    {type}
                  </option>
                ))}
              </select>
            </div>
            <div className="form-group">
              <label htmlFor="write-value">Value:</label>
              <input
                id="write-value"
                type="text"
                value={writeValue}
                onChange={(e) => setWriteValue(e.target.value)}
                placeholder={
                  writeDataType === 'BOOL'
                    ? 'true or false'
                    : writeDataType === 'STRING'
                    ? 'Text value'
                    : 'Numeric value'
                }
                disabled={writeLoading}
                required
              />
            </div>
            <button
              type="submit"
              disabled={writeLoading}
              className="btn btn-primary"
            >
              {writeLoading ? 'Writing...' : 'Write Tag'}
            </button>
          </form>

          {writeResult && (
            <div
              className={`result ${
                writeResult.success ? 'result-success' : 'result-error'
              }`}
            >
              {writeResult.success ? (
                <div>✅ {writeResult.error || 'Write successful!'}</div>
              ) : (
                <div>
                  <strong>Error:</strong> {writeResult.error || 'Write failed'}
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default TagOperations;

